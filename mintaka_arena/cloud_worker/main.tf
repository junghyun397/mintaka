terraform {
  required_version = ">= 1.6"
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 7.0"
    }
  }
}

variable "project" {
  type = string
}
variable "region" { type = string }
variable "name" { type = string }
variable "concurrency" {
  type = number
}
variable "machine_type" { type = string }
variable "archive" { type = string }
variable "lifetime" {
  type    = number
  default = 7200
}
variable "controller_ip" { type = string }

provider "google" {
  project = var.project
  region  = var.region
}

data "google_compute_zones" "available" {
  region = var.region
  status = "UP"
}

data "google_compute_machine_types" "worker" {
  for_each = toset(data.google_compute_zones.available.names)
  zone     = each.value
  filter   = "name = ${var.machine_type}"
}

locals {
  zones = [for zone, result in data.google_compute_machine_types.worker : zone if length(result.machine_types) > 0]
}

resource "google_compute_network" "arena" {
  name                    = var.name
  auto_create_subnetworks = false
}

resource "google_compute_subnetwork" "arena" {
  name          = var.name
  region        = var.region
  network       = google_compute_network.arena.id
  ip_cidr_range = "10.83.0.0/24"
}

resource "google_compute_firewall" "controller" {
  name          = var.name
  network       = google_compute_network.arena.id
  source_ranges = ["${var.controller_ip}/32"]

  allow {
    protocol = "tcp"
    ports    = ["22", "8095"]
  }
}

resource "google_compute_instance" "arena" {
  count        = var.concurrency
  name         = "${var.name}-vm-${count.index}"
  zone         = local.zones[count.index % length(local.zones)]
  machine_type = var.machine_type

  boot_disk {
    initialize_params {
      image = "debian-cloud/debian-12"
      type  = "hyperdisk-balanced"
      size  = 10
    }
  }

  network_interface {
    subnetwork = google_compute_subnetwork.arena.id
    nic_type   = "GVNIC"
    access_config {}
  }

  scheduling {
    provisioning_model          = "SPOT"
    preemptible                 = true
    automatic_restart           = false
    on_host_maintenance         = "TERMINATE"
    instance_termination_action = "DELETE"
    max_run_duration { seconds = var.lifetime }
  }

  metadata = {
    enable-oslogin         = "FALSE"
    block-project-ssh-keys = "TRUE"
  }

  metadata_startup_script = <<-SCRIPT
    #!/bin/bash
    set -e
    mkdir -p /opt/mintaka
    timeout 600 sh -c 'until test -f /tmp/arena.tar; do sleep 2; done'
    tar -xf /tmp/arena.tar -C /opt/mintaka
    cd /opt/mintaka
    nohup python3 -m mintaka_arena.arena_worker --max-concurrency "$(nproc)" >/dev/null 2>&1 &
  SCRIPT

  provisioner "local-exec" {
    environment = {
      CLOUDSDK_CORE_PROJECT = var.project
      CLOUDSDK_COMPUTE_ZONE = self.zone
      INSTANCE              = self.name
      ARCHIVE               = var.archive
      WORKER_ADDRESS        = "http://${self.network_interface[0].access_config[0].nat_ip}:8095"
    }
    command = <<-SCRIPT
      set -e
      gcloud compute scp --quiet \
        --scp-flag='-oConnectionAttempts=60' --scp-flag='-oConnectTimeout=5' "$ARCHIVE" "$INSTANCE:/tmp/arena.tar.part"
      gcloud compute ssh --quiet "$INSTANCE" --command='mv /tmp/arena.tar.part /tmp/arena.tar'
      curl --silent --show-error --fail --retry 120 --retry-connrefused --retry-delay 2 --max-time 5 \
        "$WORKER_ADDRESS/status" >/dev/null
    SCRIPT
  }

  depends_on = [google_compute_firewall.controller]
}

output "worker_addresses" {
  value = [for instance in google_compute_instance.arena : "http://${instance.network_interface[0].access_config[0].nat_ip}:8095"]
}
