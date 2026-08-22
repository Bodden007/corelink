# CoreLink

Native Rust-ядро для промышленной HMI.

**Клиент:** Avalonia / .NET 10  
**Ядро:** Rust  
**Протокол:** Modbus TCP

## Принципы

- SOLID без слоёв ради слоёв.
- Только то, что используется сейчас.
- Никаких заделов на будущее.
- Интерфейсы и trait — только при реальной необходимости.
- Читаемость важнее абстракций.
- Все ожидаемые ошибки — через `Result<T, E>`.
- `panic!`, `unwrap()`, `expect()` в production-path не используются.
- `mod.rs` только объявляет состав модуля. Реализация в нём не хранится.

## Разделение ответственности

### Avalonia

- UI / SVG.
- Rx.
- FFI к CoreLink.
- Формирует бинарный стартовый `blob`.
- Получает от CoreLink готовые значения.

Цель для UI — Native AOT.

### CoreLink

- Лицензирование и HWID.
- Modbus TCP.
- Polling / reconnect.
- Формирование Modbus-запроса.
- Разбор регистров.
- Преобразование WORD/FLOAT.
- Запись команд.
- Внутренний lifecycle.

После `Start` ядро живёт самостоятельно.

## FFI

Внешний контракт минимальный:

```text
Start(blob, ip)
Stop()
```

`blob` содержит:

```text
CRC(corelink.dll) + RegisterMap
```

Карта типов компактная:

```text
[1, 2, 2, 2, 1, 2, 1]
```

- `1` — WORD, 1 Modbus register.
- `2` — FLOAT, 2 Modbus registers.

CoreLink по карте сам рассчитывает layout и длину запроса.

```text
RegisterMap
    ↓
Modbus request
    ↓
TCP polling
    ↓
registers
    ↓
WORD / FLOAT decode
    ↓
готовые values
    ↓
Avalonia / Rx / UI
```

Avalonia не должна знать offsets, endian, reconnect и детали Modbus.

## Защита сборки

При release-сборке:

```text
BuildCounter
    ↓
вшивается в corelink.dll
    ↓
готовая DLL
    ↓
CRC32
    ↓
BuildCounter + CRC + RegisterMap
    ↓
Ed25519 signature
    ↓
поставляемый .bat
```

Private key существует только у разработчика.  
В CoreLink находится только public key.

При `Start` CoreLink:

1. разбирает `blob`;
2. проверяет подпись сборки;
3. получает HWID;
4. проверяет лицензию машины;
5. строит register layout;
6. запускает ядро.

Патч DLL меняет CRC и делает подпись сборки недействительной.

## Лицензия машины

Второй файл `.txt` создаётся самим CoreLink.

Он используется для привязки лицензии к HWID конкретной машины.

Пока build и HWID используют одну пару Ed25519-ключей. Разделение ключей добавляется только при реальной необходимости.

## Lifecycle

Внутренние состояния принадлежат только CoreLink:

```text
Created
  ↓
Configured
  ↓
Authorized
  ↓
Running
  ↓
Stopped
```

UI не управляет внутренними переходами.

## Главное правило

CoreLink должен оставаться маленьким, самостоятельным и понятным ядром.

Если решение можно реализовать проще без потери надёжности и безопасности — выбирается более простое решение.
