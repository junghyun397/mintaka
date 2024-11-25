# mintaka-renju
mintaka applies strict Renju rules while maintaining low computational cost.
Below are some examples of strict Renju rules that mintaka can handle.

## Single-line forbidden moves
![single-line-forbidden-moves](./images/single-line-forbidden-moves.png)
```text
   A B C D E F G H I J K L M N O
15 . . . . . . . . . . . . . . . 15
14 . . . . X . X 4 X . X . . . . 14
13 . . . . . . . . . . . . . . . 13
12 . . . . O O O O O O O . . . . 12
11 . . . . . . . . . . . . . . . 11
10 . . . . X . X X 4 . X . . . . 10
 9 . . . . . . . . . . . . . . . 9
 8 . . . . O O O O O O O . . . . 8
 7 . . . . . . . . . . . . . . . 7
 6 . . . . X X . X 4 . X X . . . 6
 5 . . . . . . . . . . . . . . . 5
 4 . . . . O O O O O O O . . . . 4
 3 . . . . . . . . . . . . . . . 3
 2 . . X . X . X X . . X X . X . 2
 1 . . . . . . . . . . . . . . . 1
   A B C D E F G H I J K L M N O
```
## Double-four related pseudo forbidden moves
![double-four-related-pseudo-forbidden-moves](./images/double-four-related-pseudo-forbidden-moves.png)
```text
   A B C D E F G H I J K L M N O
15 . . . . . . . . . . . . . . . 15
14 . . . . . . . . . . . . . . . 14
13 . . . . . . . . . . . . . . . 13
12 . . . . . . . . . . . . . . . 12
11 . . . . . . . O . . . . . . . 11
10 . . . . . O . . . . . . . . . 10
 9 . . . . . X X . . . . . . . . 9
 8 . . . . . . O X O . . . . . . 8
 7 . . . . . . O X O . . . . . . 7
 6 . . . . . . . . X X X O . . . 6
 5 . . . . . . . . . . . . . . . 5
 4 . . . . . . . . . . . . . . . 4
 3 . . . . . . . . . . . . . . . 3
 2 . . . . . . . . . . . . . . . 2
 1 . . . . . . . . . . . . . . . 1
   A B C D E F G H I J K L M N O
```
## Nested forbidden moves
![nested-forbidden-moves](./images/nested-forbidden-moves.png)
```text
   A B C D E F G H I J K L M N O
15 . . . . . . . . . . . . . . . 15
14 . . . . . . . . . . . . . . . 14
13 . . . . . . . . . . . . . . . 13
12 . . . . . . . . . . . . . . . 12
11 . . . O . . O . . . . . . . . 11
10 . . . . X . . X . O . . . . . 10
 9 . . . . O X O X X . . . . . . 9
 8 . . . . . . X X 3 . . . . . . 8
 7 . . . . . . O O X X . . . . . 7
 6 . . . . . X . . . . . . . . . 6
 5 . . . . O X . . . . . . . . . 5
 4 . . . . . O . . . . . . . . . 4
 3 . . . . . . . . . . . . . . . 3
 2 . . . . . . . . . . . . . . . 2
 1 . . . . . . . . . . . . . . . 1
   A B C D E F G H I J K L M N O
```
## Multiple-nested forbidden moves
![multiple-nested-forbidden-moves](./images/multiple-nested-forbidden-moves.png)
```text
   A B C D E F G H I J K L M N O
15 . . . . . . . . . . . . . . . 15
14 . . . . . . . . . . . . . . . 14
13 . . . . . . . . . . . . . . . 13
12 . . . . . . . X . . . . . . . 12
11 . . . . . . . . . . . . X . . 11
10 . . . . . O . . . 3 . . O X . 10
 9 . . O X . . X O X X X O X . . 9
 8 . . . X O O . X O 3 . . . . . 8
 7 . O . . X X 3 . . X 3 . . . . 7
 6 . . . . . . X . . . X X O . . 6
 5 . . . . O . . . . . . . . . . 5
 4 . . . . . . . . . . . . . . . 4
 3 . . . . . . . . . . . . . . . 3
 2 . . . . . . . . . . . . . . . 2
 1 . . . . . . . . . . . . . . . 1
   A B C D E F G H I J K L M N O

```
## Recursive forbidden moves
![recursive-forbidden-moves](./images/recursive-forbidden-moves.png)
```text
   A B C D E F G H I J K L M N O
15 . . . . . . . . . . . . . . . 15
14 . . . . . . . . . . . . . . . 14
13 . . . . . . . . . . . . . . . 13
12 . . . . . . . . . . . . . . . 12
11 . . . . . . . . . . . . . . . 11
10 . . . . . . O O . . . . . . . 10
 9 . . . . . 3 X X 3 . . . . . . 9
 8 . . . . O X X X X O . . . . . 8
 7 . . . . O X X X X O . . . . . 7
 6 . . . . . 3 X X 3 . . . . . . 6
 5 . . . . . . O O . . . . . . . 5
 4 . . . . . . . . . . . . . . . 4
 3 . . . . . . . . . . . . . . . 3
 2 . . . . . . . . . . . . . . . 2
 1 . . . . . . . . . . . . . . . 1
   A B C D E F G H I J K L M N O"
```