use crate::notation::pos;

pub const EMPTY_HASH: u64 = 0;

pub const HASH_TABLE: [[u64; pos::BOARD_SIZE]; 2] = [
    [
        0x359838E6A0B78035,
        0x142BCE1982708FE4,
        0x1E2E2505809D9DFC,
        0xA027A719E5EF4432,
        0xBB540BC6469EBDCE,
        0xDA04FE6BB6598A22,
        0x51C999A2F2BD0B64,
        0xE8FF76D7FD2CA76B,
        0x7E8298895C9349B0,
        0xB054E08820F77F4D,
        0x5AB19949772327CA,
        0x70B827A2A71AF34C,
        0x5FA5D494F1C5BF34,
        0xF7F40721CC35AA2E,
        0x17E51E81BFA9C704,
        0x214B98C9B50884B1,
        0x25B2B4E8D04F5EE6,
        0xE1E7821F055B3CC6,
        0xCE74CF06EBEB7591,
        0xAC95182B7A745E96,
        0x6035AC0475656D4B,
        0xF77B450F158C4DA4,
        0x8E4A5094B2E17C53,
        0x4F1146DCD1674472,
        0xC73D08C66F270D48,
        0x6AFF6366EEBD763C,
        0x9183B411BD27EBB2,
        0x03EE3D5FD1F2E9AC,
        0xB125DBED944E2793,
        0x2AFF19D12A74F3A0,
        0x118A7104410EF1FA,
        0xF74512FCFD4D63F5,
        0xF3BC8ACFC5B62F3B,
        0xA7B01E7B8CFFFC21,
        0xA0517C8C97E73C08,
        0xC15F546703C4DB61,
        0x15B55570206C4FFE,
        0xC264A77F1E2DEDDC,
        0x26E5CB46380DABB4,
        0xE9827E9E0825466D,
        0xF7E1F54CE93E9E81,
        0x3D6EF331C12FA71B,
        0xFCF4180BACC8577A,
        0xB5FB49992CA513EF,
        0x5DCA644723ED4568,
        0x96BBDB2A64156AFE,
        0x6240756F97851F23,
        0x68C4535E878B4C26,
        0xFD92EA4BCF50E102,
        0x748472CA8047F573,
        0x7605AD89311B9FD0,
        0x0B39B735AAE0B73F,
        0x2579752D29F03CA9,
        0x79EA3F163C7CDB2E,
        0xA838B45AE8EBFF54,
        0xECC9C55658AADC88,
        0x0A9AA86DC2FCC2B3,
        0x26A71A2505AA04B2,
        0x10D14745DBEE08E5,
        0xBAB43D594210AF4B,
        0xC904A9ACAE1C01C8,
        0x658639DEFF08C54C,
        0x7DE6B0942C47EC4C,
        0xA80431B254ADEAED,
        0xBD09B76257A225C7,
        0x3C24EE34EBFB871C,
        0x57ACE948BDB09630,
        0xF6FF9887FB76BDB8,
        0xBCBAD44EBA40EDCD,
        0x8A242880BE9EB785,
        0x5CCDC04B839BEF91,
        0xE80190B481E46D74,
        0xF91076313590A001,
        0x45FA2732897273E2,
        0xB94354A7E74E04B3,
        0x43CED9517622F5BE,
        0xB3D28AF2E41C3DDA,
        0x5AFCDEF9DBDF58D3,
        0x203E3AAA17F7DFBD,
        0x0959CE145B0E893C,
        0x2F223699235F55DF,
        0x020E1FD1F6926CF5,
        0xC5065C08F610EAB8,
        0x8416970B069980AA,
        0x8C3C24A88E5A14AD,
        0xDA7126D6252BBEAF,
        0x707F1919DBC5DBD2,
        0x2F3560FC2C656800,
        0xA929F5DBD5A71215,
        0xEB332D23035F2A61,
        0xC2580A8432DB815A,
        0x3DB0EBDDF2E03BAA,
        0x8D3B72AE1E283E53,
        0x0C79854FB01D1384,
        0x1FA38B4D56062EE2,
        0x5A7BAEB01DBF715C,
        0x450FD49105B670F0,
        0x61CF52BAC182B44C,
        0x100E08E1A768A1E9,
        0xCC9E80CDA6D99034,
        0x8A210191433DF7BB,
        0xF054B1DD0E61F8BF,
        0xDBCF1E0EB720A5DF,
        0x5C2B97A35399BB63,
        0x49548F05777E1DE4,
        0xC7C856470F81DB78,
        0x43574417BB9A9DB1,
        0x346985B211E64AE9,
        0x580F2C27F1C18665,
        0xF8083B97A2035FB9,
        0x91FEC29A22DA979D,
        0xF2BA6CDD4BDE9891,
        0x664600DDA333EAB2,
        0xDB0DDC4A00267104,
        0x5C7B55573F145E62,
        0xE5509D2A92161A87,
        0x286C6405294A6E5D,
        0x5A1E83CC4E1C1B77,
        0xDAE86CACF70707DA,
        0x0CFF7062F5B93B4B,
        0xAC45A652614DFAA0,
        0x7759DC31E11D9996,
        0xED431EB7101E231F,
        0xDF40A22A4D6DF1F6,
        0xEBD7244BCD443C7B,
        0xF361FA579C5EDC70,
        0x63D4FEB8B690FDAE,
        0x1E642711D76A1FCF,
        0x64F5E93E9BE4CBC3,
        0x5F09352085BECE46,
        0x05F4D96049710AB0,
        0x80D109A478C8CA76,
        0x2CD1F4BA22C9869D,
        0x23E8D16340F624B8,
        0x6CEBFD9B144D0F8C,
        0x62601B740D1E9377,
        0x72B335AB081DFE35,
        0x30741136AE143510,
        0x54EBACFF581AD158,
        0x02A694C28FB7B480,
        0xD80D053A8369662C,
        0x19C90F05E1D5A311,
        0x9C12A45FC46215B3,
        0x262BD817DB052FB9,
        0x4006A4DF90A37F5C,
        0xD7B5FC40C554B0BB,
        0xB0910040B3096C92,
        0xA37C37B4CEB141DD,
        0x66AE8CA68BE9D562,
        0xA3CC5D18FF4D9B1A,
        0x2CB25CFDD730E968,
        0x1C502F8790DCFA2D,
        0x912F2A6DEBA35FAF,
        0xDCD56189E69AF82E,
        0xDC6CD6B7B8B0342A,
        0x1C1DA1ADE6CA670F,
        0x7B6B84A5B6C5365A,
        0x87F3952B2C7EC15C,
        0x77CF5DE12EE6D363,
        0x12447699C556816C,
        0x1BD93302B581AC3E,
        0x9EC75BA98A3769D0,
        0xC63CD0193C99A6B6,
        0x143A6F6374EF59E9,
        0xBA4560953F2384D1,
        0x17C341CB00E569E0,
        0xCA52D656DDD4C481,
        0xDF5AA0E680B4F46A,
        0x831084E50A7114C3,
        0x1D7A7347552FA995,
        0xBF8CE3F393E69714,
        0xB0BE06BF29867450,
        0x8C78096361E99D12,
        0x747B06ED252E110F,
        0xD554BCAC72CA5A56,
        0x375AA9F7C9B13352,
        0x2E132117A8218DB0,
        0x4D2BFCB1E6A58916,
        0x4525F7DCAE2B3C36,
        0x48A7DE949EF19681,
        0xAEFE47A4B66BA768,
        0xE9E6F4B70617210D,
        0x93617FA87CD5CC69,
        0xD6CDBCAD9129BBC1,
        0x82B14CF7542DAEE5,
        0x40A45A060485571F,
        0xA37A151AF504B79B,
        0xE3EF1D0DB600D631,
        0xA9E9A8687B9EF820,
        0xC5E6A339C0490DD8,
        0x6386B3B3D1A79673,
        0x6F48A8C40350E991,
        0xF883BB5DF49361D5,
        0x568DE11BF27C5956,
        0x0B5A27D4B140B781,
        0xFC0DF15D4259FABD,
        0x8352B14F6CD0B05D,
        0x62823E38D2B4F022,
        0x5BF07059533CF43B,
        0xE16AEB530CC0532D,
        0x896AB7CE64CCE205,
        0xB606D5D183123D3C,
        0x293B83F99E262D0A,
        0xD13F38B3B8FE86D3,
        0x82E95F12E6EF3C85,
        0xF8588EC98CE05D14,
        0xB028342B9710BCE8,
        0xC0185936E809C2D1,
        0x995F55A9299A24C3,
        0x04400C43D89C6316,
        0xC10F2AF6C1907CE4,
        0x766E633C9D3D0E36,
        0xC6D770DBA8904825,
        0xA96B1A4551D639CB,
        0x2A9C9AADFDA89300,
        0xF6E2A80A37D8E9BD,
        0x0FA37DDAFE0644D9,
        0x127BB428055CBB66,
        0xED326A2AB2DEABA4,
        0xBD236CEA62D85667,
        0xC5DF60D9D000584B,
        0x73534049D5B5E11E,
        0xD016171A396DA1E5,
        0x907AD7E77A7519D1,
        0x09BDE1C5E930F042,
    ],
    [
        0xF340F1CBF42C2C73,
        0xA070EA0A203E92D0,
        0xD27DF0EB429DF031,
        0x4FC52B32A9114C4C,
        0x298DFD5913543268,
        0x9E2D393580C45AC9,
        0x9172661EE708FB78,
        0xED5F4A31CC8E34E5,
        0xFC5FD7DD25CB2514,
        0x3D086966D0F1E119,
        0xD20156DD920B98DE,
        0x8CC09E12DD1DA8A8,
        0x0E7806A8B1EBB5B1,
        0x7860A3CF85CF2CB7,
        0xDAF50AAFAF846E89,
        0x3606016A7F79A227,
        0xDDFA113A3B292A32,
        0xEF017590F0C3761C,
        0x528E73A03ED28E94,
        0x3FA5368F14420D34,
        0x767022B7B6F01A98,
        0xA45325DBD5F8C339,
        0x382A1E061119706B,
        0xD2A7223CF55CC85E,
        0xC526AE5AD25B40AE,
        0x54BB1CA1F64B985C,
        0x5164A9158754EE61,
        0x49609FD92530BE9B,
        0xFAEDA863F486991F,
        0xF7D552E65B17310E,
        0xCA682CAB6FB50D36,
        0x0BFF643172FAB818,
        0xE7FE8934FFD40791,
        0x4F27424BA3EF45F1,
        0xC3E338A58464C491,
        0xDFFA0EB132E9B021,
        0x4B57911BC8CEC841,
        0xB752306930F4847A,
        0xB0588EDAD48E1586,
        0x975EABD861146CDC,
        0xD94744BE146D1CC0,
        0x024DDDC38EF0733F,
        0x0C79910156F42842,
        0x2AB8EE92B24608A8,
        0xAAC3BE1869B2D24E,
        0x6FD1E8348E03FD0B,
        0x39CC99BB9E8CFC43,
        0x064A41D8501EFA88,
        0xCE1C0CAE7D03FF3C,
        0xBEFE200E66295086,
        0x2BEE2CB44BC87AA3,
        0xF1D976B1185846EF,
        0xC67C28472D2C3198,
        0x2A1244047DBDA32D,
        0xBDE7D9F4E2D667E3,
        0xA370C5E3FE7DAAFA,
        0xE7E158D3DFB77EF6,
        0x5281CA10AEE3AAEC,
        0x057BB550A10B2EB3,
        0xF80D70C4E010CDC0,
        0x9E71A4DE9C152DDA,
        0xB2BA40BCC1D7F7E7,
        0x4C0E3F315CF9EA49,
        0xC8DC2962EA1FC130,
        0x4CB49197C165132C,
        0x9E86622E7111990E,
        0xCBD57DF02E0CE929,
        0x87EEE3CF38769286,
        0x671C6957D5AF4E3D,
        0x7C0E76D432AADEFD,
        0x20957637081AEB4E,
        0x49BEA54C0B7C5BA1,
        0x056E30821827BB25,
        0x30A4E1CC3BCB2FC0,
        0xC062C9BC95F0283C,
        0x609DE664BDBE2B68,
        0x1780C611557FDCD3,
        0x69726CD4EB53504E,
        0xE85144948C094106,
        0x2583690418846344,
        0xC2AFF1D8615361EF,
        0xC2578E78E5456369,
        0xE58D02CF5583E182,
        0x52EBA4E72DC856C1,
        0xF44BA4E3A3BA0C53,
        0xA23E51DEA4347843,
        0xC0979A98115C4085,
        0x22CD3CBB900B1819,
        0xDAFB5B7F39356B91,
        0x278DDD81F80061BF,
        0xD772D40F98987DE0,
        0x196FCB5722F0D27A,
        0x4773D8785FF63DF2,
        0xCDA9C6DCD63C65B3,
        0x0C3B05930E43608D,
        0x4FC2AEEB5BCB3467,
        0x5F74D7252951994D,
        0x3D856E927A45CB96,
        0xE074B42CE6D177AA,
        0x459215A35115D360,
        0x8D291151E38D58FC,
        0xFB13456FF5972665,
        0xBDA3463103C1F91E,
        0xC2CF7921C6E92DAF,
        0x29E66470A140FF52,
        0xB5CD1A460B8CFA98,
        0x357A461C3CA344DB,
        0xA9D0B1C083DC2E2B,
        0x75999C2807DB2D40,
        0xA1488539E102260A,
        0x4FDD1840FECE3CCC,
        0x5529599C4F6FDD9C,
        0xDF76325B6FA8F2B9,
        0xFAE891CEE9C4109D,
        0xBDA53DD1698BA55B,
        0x36832EA3182B440C,
        0x1C5F550018AB53A3,
        0x965C3784153CCB5D,
        0x4A2423DC1490845B,
        0x991618BE26C6C8E2,
        0x6120AE057AEC83BD,
        0x0BC6C10A4E58EA9D,
        0x693B83A073A1D7EE,
        0x16312B6EA4CF35C7,
        0x7F0A792962B617C8,
        0x16946BD96F6FC2D9,
        0x55A059392AD37030,
        0xE720A76E4C8F064F,
        0x414EA808A68EEC73,
        0x7A03C6C2875E710E,
        0x70A98463370F0145,
        0x54A60EDD6D8C3551,
        0xF2A6CC14D53A2B7D,
        0x2FFFDC99B8199FA4,
        0x19A96A2BB6B666CD,
        0x22CCA86211CECB2E,
        0x694FAE36283D36EB,
        0x90430DD37500F128,
        0x525681CC469872CC,
        0x615F084AFD0DB8FE,
        0x8E53F320CD7B77EF,
        0x4A6B5A3849A0531F,
        0xC52C29855DF00B7A,
        0xF9D54682548458B3,
        0x415FF603BDE6F706,
        0xA98C7CA20DDC3973,
        0xA32FEF3E564556A7,
        0x92D89A3C86CFF58F,
        0xDCC8B137882F345F,
        0x66A1DFC25A2F63FC,
        0xDCEF2F890F0DC99F,
        0xC33C89E8E648D3C0,
        0x2511E4A104E1FF42,
        0xCBBE61A7B9542710,
        0x597068741295F72A,
        0xF0F8A6E390145A4F,
        0x82A15441EB250943,
        0xA73DF4DAB74D54F7,
        0x43F49FAD3ED4233A,
        0x2D7D622DA3ED49DB,
        0x54B4BDDA535AE1EC,
        0x643523D14F8C5440,
        0x09BACC02E978B484,
        0x6E809BC70A4F17A9,
        0x17B04B30D4409103,
        0x62F730D6F19FCC1B,
        0xD4DE2C09C81FED56,
        0xA59111B21F923265,
        0xCC184DFA916C636E,
        0x1B43FFE3C152FEF6,
        0xE85BD9ECEAC1AC99,
        0xD81EC9A574C65942,
        0x755400F88FC4F2A6,
        0xD9744974DB2D06B2,
        0x8BAA9C7237287C6F,
        0x1798DE45B726BAB1,
        0xE37C456E4D0CCF07,
        0x8FCFC54C88412C32,
        0xBBE95601E1E8FC59,
        0xA660255ED8D76A28,
        0x160EABB33B76B6BE,
        0x34ED9ADB612413B4,
        0x273B5B01CF4C5B07,
        0x833E1A223F060F38,
        0x3C40A154596279F6,
        0xB4D0794AA3FA8692,
        0xC05772453EB3C482,
        0x547D264215A4A7D8,
        0x51B2425A5CB2DFAD,
        0x16F1BEC7734D76E0,
        0xD8C67331467525F1,
        0x2D593B33EEBF75C7,
        0x7879025925DF25F1,
        0x6121070530EC1C30,
        0x50DC66164CFB61C1,
        0xF46CCA74D8623534,
        0x5AEED758E44B4FC4,
        0x7DB8339331E30DE7,
        0xFCE9E1AC242BC1BA,
        0x60321FB2B010D715,
        0x7E959DDC03F03576,
        0x3CC8B25F299D9EF1,
        0x8262F903F1C2A826,
        0xDFFDB0D820A9EEA8,
        0x238E8EB0F06750F2,
        0x6873CD57F1E96D39,
        0x90845FE18B24CE90,
        0xBDC8222FD8E51AFD,
        0xC9C843DB24158A19,
        0x40EBB067413B7A95,
        0x6D10E0D3DA2E0524,
        0xE1234CCE909C93DE,
        0xA549DFB640296D08,
        0x61DC8BF4C5B365EC,
        0xD05E5CAAB67CDACA,
        0xE3F17807FED2A759,
        0xD02882FE884C1725,
        0x3258A8397C5214D7,
        0x60EF40C82C005D2F,
        0x83143CCA7EFEAD35,
        0x05808401E26E939E,
        0xF09D1E2A300FD481,
        0xA04878275B0F36C1,
        0x7B99385FE30C9AB6,
        0x4B88D34726551869,
    ]
];