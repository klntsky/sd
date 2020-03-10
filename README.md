## Запуск

```bash
cargo test
cargo run
```

## Детали реализации:

Почти все символы разрешено использовать в строковых литералах без кавычек.

```bash
$ echo *?
*?
```

Рядом стоящие строки обрабатываются не как в bash:

```bash
$ echo "a""b"
a b
```

Вывод обрабатывается не потоково, а по завершению. Тем не менее, вывод последней команды стримится в консоль.

Комментарии не поддерживаются.

Перменные окружения видны внешним командам.

Шелл не делает различия между stdout и stderr.

## Примеры

```
$ cal | grep 12 | sed s/12/33/
 9 10 11 33 13 14 15
$ echo foo | wc
        1       1       3
$ pwd
/home/me/itmo/ppo/sd
$ foo=ex
$ bar=it
$ $foo$bar
```

### Примеры с grep

```
$ cat tmp
foo
bar
foo
baz
$ grep foo tmp
foo
foo
$ grep 'ba.' tmp
bar
baz
$ grep foo -A 2 tmp
foo
bar
foo
baz
$ grep bar -A 2 tmp
bar
foo
$ grep bAr tmp -i
bar
$ grep '(bAr|fOo)' tmp -i
foo
bar
foo
```

```
$ cat tmp2
barbar
barbaz
foo bar baz
barfoo foobar
$ grep foo tmp2 -w
foo bar baz
$ grep "bar baz" tmp2 -w
foo bar baz
$ grep "bAr bAz" tmp2 -w -i -A 2
foo bar baz
barfoo foobar
```

Особенности реализации: парсер аргументов ожидает ключи `-i` и `-w`, которые не имеют аргументов, поданными в конце.
Альтернативный способ передачи ключей: использование `=` (напр. `grep '-i=true' ...`).

## Обоснование выбора библиотек

`regex` - нет достойных альтернатив для rust.
`dia-args` - минималистичный и простой интерфейс, мало зависимостей по сравнению с альтернативами, маленький размер пакета.
