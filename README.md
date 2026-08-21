# CoreLink — архитектурный устав

**Проект:** `Bodden007/corelink`  
**Назначение:** защищённое native-ядро промышленной HMI  
**Клиент:** Avalonia / .NET 10  
**Основной протокол первого этапа:** Modbus TCP

## 1. Назначение

CoreLink — небольшое, понятное и детерминированное Rust-ядро, являющееся единственным владельцем коммуникации с PLC.

Ядро отвечает только за текущие реальные задачи:

1. получение конфигурации от .NET через FFI;
2. валидацию конфигурации;
3. проверку HWID и Ed25519;
4. владение Modbus TCP-соединением;
5. внутренний polling PLC;
6. преобразование Modbus-регистров в значения;
7. хранение последнего подтверждённого snapshot;
8. выполнение команд записи с приоритетом над следующим Poll;
9. timeout и reconnect;
10. небольшой стабильный C ABI для .NET;
11. детерминированный lifecycle и освобождение ресурсов.

Главный критерий архитектурного решения:

> Если компонент не решает конкретную существующую задачу текущей версии CoreLink, он не создаётся.

---

## 2. Основные архитектурные правила

- SOLID применяется без догматизма.
- Никаких слоёв ради слоёв.
- Ничего не создаётся «в запас».
- Никаких дополнительных абстракций без конкретной необходимости.
- Читаемость и очевидность потока важнее архитектурной декоративности.
- Простое решение, полностью закрывающее текущую задачу, предпочтительнее универсального и сложного.
- Не создавать инфраструктуру только потому, что «так принято» или «может пригодиться позже».

## 3. SOLID без догматизма

### Single Responsibility

Компонент имеет одну понятную ответственность. Не допускается God Object, одновременно выполняющий FFI, безопасность, Modbus, декодирование и lifecycle. Одновременно запрещается искусственно дробить одну понятную ответственность на множество типов.

### Open/Closed

Не проектировать заранее extension points для функций, которых сейчас нет.

### Liskov Substitution

Полиморфизм не является целью. Если нет нескольких реально взаимозаменяемых реализаций, соответствующая абстракция не создаётся.

### Interface Segregation

Trait создаётся только при реальной необходимости. Не создавать trait автоматически для каждой структуры или сервиса.

### Dependency Inversion

Граница создаётся только там, где она реально существует. FFI является реальной границей. Modbus-коммуникация является отдельной ответственностью. Дополнительные Application / Domain / Infrastructure границы без задачи не нужны.

---

## 4. Никаких слоёв ради слоёв

CoreLink не строится по Clean Architecture, Onion, Hexagonal или другой многоуровневой схеме только ради шаблона.

Не создавать автоматически:

- Domain;
- Application;
- Infrastructure;
- Services;
- Providers;
- Repositories;
- Factories;
- Adapters;
- Managers;
- Utils.

Каждая такая сущность должна быть оправдана конкретной текущей ответственностью.

## 5. Никаких заделов на будущее

Без отдельного требования не создавать:

- OPC UA;
- MQTT;
- gRPC;
- REST;
- WebSocket;
- универсальный `ProtocolProvider`;
- `TransportFactory`;
- Plugin API;
- DriverFactory;
- динамическую загрузку протоколов;
- конфигурационный DSL;
- JSON-карту регистров;
- runtime-переконфигурацию;
- hot reload конфигурации.

Сегодня CoreLink работает с Modbus TCP — реализуется Modbus TCP. При появлении нового реального требования архитектура пересматривается отдельно.

## 6. Читаемость

Основной поток должен читаться сверху вниз:

```text
configure
↓
validate
↓
verify
↓
start
↓
poll / write
↓
snapshot
↓
stop
```

Избегать бессодержательных имён `Helper`, `Utils`, `Manager`, `Provider`, `Processor`, `Handler`, если имя не объясняет ответственность.

---

## 7. Граница .NET ↔ Rust

.NET отвечает за:

- UI;
- View/ViewModel;
- Rx;
- формирование конфигурации конкретной установки;
- присвоение логических ID параметрам;
- отправку операторских команд;
- получение результатов CoreLink.

Rust отвечает за:

- безопасность;
- рабочую конфигурацию ядра;
- Modbus;
- polling;
- timeout;
- reconnect;
- декодирование;
- запись;
- сериализацию операций;
- хранение последнего snapshot.

.NET не открывает Modbus TCP, не выполняет reconnect и не управляет физическими Modbus-транзакциями.

## 8. Конфигурация через FFI

CoreLink не загружает карту регистров из JSON. При запуске .NET формирует конфигурацию установки и передаёт её Rust через FFI.

Обязательная последовательность:

```text
core_create
    ↓
передача конфигурации
    ↓
validate
    ↓
core_start
    ↓
HWID + Ed25519
    ↓
Authorized
    ↓
запуск коммуникационного контура
```

Порядок не меняется.

## 9. Конфигурация соединения

Передаются только реально необходимые параметры:

- IP;
- Port;
- Slave ID.

IP — единственное строковое значение рабочей конфигурации. Остальная конфигурация передаётся бинарными числовыми структурами.

## 10. Описание параметров

Каждый логический параметр содержит минимально необходимую информацию:

```text
ID
Register Area
Address
Data Type
Word Order
```

Например:

```text
ID        = 10
Area      = Input
Address   = 100
DataType  = Float32
WordOrder = LoHi
```

Rust самостоятельно определяет количество физических регистров из типа:

```text
UInt16  → 1 register
Int16   → 1 register
UInt32  → 2 registers
Int32   → 2 registers
Float32 → 2 registers
```

Не передавать данные, которые CoreLink способен однозначно определить самостоятельно.

## 11. Логический ID

Основной идентификатор параметра — числовой ID. Строковые имена технологических параметров через FFI не используются.

После конфигурации Rust знает соответствие:

```text
ID
↓
Area
Address
DataType
WordOrder
```

Физические Modbus-адреса после запуска не должны использоваться UI.

## 12. FFI-структуры

Все структуры FFI имеют стабильный layout через `#[repr(C)]`.

На ABI-границе используются фиксированные примитивные типы: `u8`, `u16`, `u32`, `u64`, `i16`, `i32`, `f32`, `f64`.

Не передавать через FFI:

- `Vec`;
- `HashMap`;
- Rust `String`;
- Rust references;
- trait objects;
- Rust errors;
- managed objects;
- callbacks без реальной необходимости.

`area`, `data_type`, `word_order`, `status` передаются фиксированными числовыми значениями и валидируются Rust.

## 13. Конфигурация immutable после запуска

После запуска рабочая карта CoreLink не изменяется. Для другой конфигурации ядро корректно останавливается и конфигурируется/создаётся заново.

Не допускается runtime-изменение адресов, типов, word order, состава параметров и параметров соединения.

## 14. Валидация

До проверки безопасности полностью проверяются:

- ID и конфликты ID;
- Modbus address;
- register area;
- data type;
- word order;
- диапазоны многословных значений;
- параметры подключения;
- внутренние противоречия конфигурации.

При ошибке CoreLink не переходит в рабочее состояние. Частично валидная конфигурация запрещена.

---

## 15. Безопасность

После успешной валидации выполняется:

```text
Obtain HWID
↓
Verify authorization data
↓
Verify Ed25519 signature
↓
Authorized?
```

До успешной проверки безопасности запрещается:

- создавать рабочий Modbus client;
- открывать TCP-соединение с PLC;
- запускать Modbus worker;
- запускать poller;
- выполнять Read;
- выполнять Write.

При отрицательном результате CoreLink остаётся Locked.

Проверка HWID/Ed25519 является частью запуска и не выполняется на каждом Poll. Runtime-перепроверка вводится только при отдельном требовании.

---

## 16. Один владелец Modbus

На один CoreLink существует один владелец Modbus-соединения. Только он создаёт соединение, читает и пишет socket, выполняет request, уничтожает соединение и выполняет reconnect.

Никакая другая task/thread не обращается к Modbus socket напрямую.

## 17. Read и Write сериализованы

Read и Write никогда не выполняются параллельно на одном соединении. Все Modbus-транзакции проходят через одного владельца.

Предсказуемость важнее потенциального выигрыша от параллельного Modbus TCP.

## 18. Poller принадлежит Rust

После успешной авторизации Rust запускает внутренний коммуникационный цикл.

.NET не инициирует физический Modbus Poll каждые 500 мс.

```text
Authorized
↓
Start Modbus Worker
↓
Start Poller
↓
PLC
```

## 19. Poll plan

После получения конфигурации Rust может объединять соседние регистры в эффективные блоки чтения.

Например:

```text
ID 1 → Input 100-101 Float32
ID 2 → Input 102-103 Float32
ID 3 → Input 104 UInt16
ID 4 → Input 105 UInt16
```

может быть прочитано одним запросом `Start=100, Count=6`.

Оптимизация разрешена только если она проста, очевидна и реально уменьшает число Modbus-запросов. Не создавать сложный optimizer.

## 20. Декодирование принадлежит Rust

Rust преобразует Modbus words согласно конфигурации: `u16`, `i16`, `u32`, `i32`, `f32` и поддерживаемый word order.

UI не собирает `Float32` из двух `ushort`. После запуска физическое представление Modbus остаётся внутри CoreLink.

---

## 21. Latest Snapshot

CoreLink хранит последний успешно полученный технологический snapshot.

Минимально snapshot содержит:

```text
Sequence
Timestamp
Quality / Status
Values
```

`Sequence` изменяется только при появлении нового подтверждённого состояния.

## 22. Latest-value semantics

CoreLink не хранит историю каждого Poll. Для HMI используется принцип: требуется последнее актуальное подтверждённое состояние.

Если между двумя обращениями UI произошло несколько Poll, UI получает последнее состояние.

Не создавать очередь исторических snapshots, тренды или архив без отдельного требования.

## 23. Получение данных .NET

Rx в .NET работает с периодом 500 мс, но не запускает Modbus Poll. Он спрашивает CoreLink, существует ли snapshot с более новым `Sequence`.

```text
Rx 500 ms
↓
CoreLink.GetLatest(lastSequence)
↓
NewData / NoData / State
```

Если `Core sequence > lastSequence`, возвращается новый snapshot. Если sequence не изменился — новых технологических данных нет.

.NET polling является polling готового состояния CoreLink, а не polling PLC.

## 24. UI получает результат по факту

UI не должен видеть внутренний ход Modbus-операции. UI не интересует начало TCP connect, текущий Read, номер reconnect или внутренний этап timeout.

Основной API возвращает фактическое состояние CoreLink, например:

```text
NewData
NoData
Disconnected
Faulted
```

## 25. Timestamp принадлежит Rust

Timestamp создаётся CoreLink в момент успешного получения данных от PLC. .NET не заменяет его временем чтения snapshot из ядра.

## 26. Старые значения при потере связи

Последний успешный snapshot может сохраняться, но его качество должно однозначно показывать, что данные устарели.

```text
Values    = last successful values
Timestamp = last successful timestamp
Quality   = Disconnected
```

Старые значения не выдаются как новые.

---

## 27. Timeout

Отказ от синхронного `PollOnce` через FFI не означает отказ от timeout внутри Rust.

Timeout ограничивает как минимум:

- connect;
- Modbus read;
- Modbus write.

Зависший socket не должен навсегда блокировать единственного Modbus worker.

Внутренние timeout не выносятся в UI без необходимости.

## 28. Reconnect

Transport error, protocol failure или timeout, после которого соединение нельзя считать надёжным, инвалидируют текущее соединение.

```text
Request
↓
Transport / Protocol / Timeout failure
↓
Drop Modbus client
↓
Drop TCP stream
↓
Disconnected
↓
Reconnect according to policy
```

Не продолжать использование соединения с неизвестным состоянием.

Reconnect policy должна быть простой и централизованной. Не создавать сложный backoff framework без реальной необходимости.

Потеря PLC является штатным состоянием HMI и не завершает CoreLink.

---

## 29. Запись

.NET передаёт:

```text
Parameter ID
Value
```

Rust выполняет:

```text
ID
↓
Register definition
↓
validate value/type
↓
encode
↓
Modbus Write
```

Физический address в рабочем Write UI не передаёт.

## 30. Приоритет записи

Write имеет приоритет перед следующим Poll, но уже начатая Modbus-транзакция никогда не прерывается.

```text
Active transaction
↓
finish
↓
pending Write?
├─ YES → Write
└─ NO  → Poll
```

Если Write пришла во время Poll, она ждёт завершения Poll и выполняется следующей.

Не открывать второе соединение для Write.

## 31. Очередь записи

Write queue должна быть bounded. Бесконечная очередь запрещена.

Размер определяется фактическими потребностями HMI, а не выбирается большим «на всякий случай».

## 32. Poll не должен голодать

Приоритет Write не означает бесконечное вытеснение чтения. После обработки ожидающих операторских команд должен выполняться Poll.

Не создавать отдельный сложный scheduler framework ради этой политики.

## 33. Подтверждение записи

Успешная Modbus Write означает только успешное завершение транзакции записи. PLC остаётся источником технологической истины.

После Write следующий Poll получает фактическое состояние PLC:

```text
UI command
↓
Write
↓
Success
↓
Poll
↓
PLC actual state
↓
New Snapshot
↓
UI
```

---

## 34. Ошибки только через Result

Все ожидаемые ошибки обрабатываются через `Result<T, E>`.

В production-path запрещены `panic!`, `unwrap()` и `expect()` как механизм обработки штатных ситуаций.

Timeout, отсутствие PLC, некорректная команда, ошибка конфигурации и protocol error являются контролируемыми состояниями.

Rust Error не пересекает FFI boundary. Через FFI возвращаются стабильные числовые статусы, например:

```text
Success
NoData
NotRunning
Unauthorized
Disconnected
Timeout
Busy
QueueFull
InvalidConfiguration
InvalidParameter
InvalidValue
ProtocolError
InternalError
```

Конкретный набор фиксируется при реализации.

## 35. Unsafe

Основная логика — safe Rust.

`unsafe` допускается только там, где действительно необходим, прежде всего на минимальной FFI-границе. Unsafe-код должен быть локализован и проверять входные pointers и lengths.

---

## 36. Worker и runtime

На первом этапе существует один понятный владелец Modbus-коммуникации.

Tokio допустим, если реально упрощает TCP, timeout, channels и lifecycle. Tokio не является архитектурной целью.

Runtime создаётся один раз и имеет явного владельца.

Каждая thread/task имеет владельца, причину существования, момент запуска, механизм остановки и контролируемое завершение. Detached tasks запрещены.

## 37. Lifecycle

```text
Created
↓
Configured
↓
Validated
↓
SecurityCheck
↓
Authorized
↓
Running
↓
Stopping
↓
Stopped
```

При ошибке безопасности: `SecurityCheck → Locked`.

Обычная потеря PLC не переводит весь CoreLink в глобальный `Faulted`.

## 38. Stop

```text
Stop accepting new commands
↓
Stop poller
↓
Stop Modbus worker
↓
Close connection
↓
Release runtime resources
↓
Stopped
```

`destroy` выполняется после корректной остановки либо сам гарантирует безопасное завершение принадлежащих CoreLink ресурсов.

---

## 39. Минимальный C ABI

Публичный API должен быть маленьким. Концептуально:

```text
core_create
core_configure
core_start
core_get_latest
core_write
core_stop
core_destroy
```

Новая функция появляется только при конкретной необходимости.

FFI — это внутрипроцессная граница, а не REST/gRPC/RPC framework.

## 40. Состояние CoreLink

CoreLink хранит только необходимое ему состояние:

- configuration;
- authorization state;
- connection state;
- Modbus worker state;
- latest snapshot;
- sequence;
- write queue.

Не создавать параллельную бизнес-модель установки внутри Rust. PLC остаётся источником технологической истины.

---

## 41. Структура исходников

Не создавать отдельный crate без реальной самостоятельной границы. Разделение на crate — архитектурное решение, а не способ раскладывания файлов.

На первом этапе предпочтительна небольшая структура примерно такого уровня:

```text
corelink
├─ ffi
├─ core
├─ security
└─ modbus
```

Даже эта структура не является догмой. Файл/module создаётся только когда появляется соответствующая реальная ответственность и это повышает читаемость.

Не дробить код ради количества строк.

## 42. Trait и Factory

Trait создаётся только если:

- существуют несколько реализаций;
- требуется реальная подмена в тесте;
- существует настоящая архитектурная граница.

Не создавать пару `Trait + Impl`, если существует одна реализация и абстракция ничего не даёт.

Factory/Builder/Provider/Resolver не создаются, если объект можно понятно создать напрямую.

## 43. Никакого глобального mutable state

Configuration, authorization, snapshot, connection, Modbus client и write queue принадлежат экземпляру CoreLink.

Глобальное изменяемое технологическое состояние запрещено.

## 44. Минимальный public API

Внутренние структуры и функции остаются private, пока нет реальной причины выставлять их наружу. `pub` не используется автоматически.

## 45. Комментарии

Комментарии объясняют причину неочевидного решения, invariant, ограничение протокола, причину unsafe или особенность оборудования. Комментарии не должны пересказывать очевидный код.

## 46. Основные invariants

```text
No communication before authorization.
Only one owner of Modbus connection.
Only one active Modbus transaction.
Write has priority over next Poll.
Active transaction is never interrupted.
Snapshot sequence changes only for new data.
Configuration is immutable while Running.
PLC is the source of technological truth.
```

Эти правила должны быть очевидны непосредственно из кода.

## 47. Не оптимизировать заранее

Сначала:

```text
correct
↓
readable
↓
measured
```

и только затем optimized.

Очевидное объединение соседних Modbus-регистров допустимо. Сложные lock-free структуры, memory pools, custom allocators и другие оптимизации без измеренной проблемы не применяются.

## 48. Производительность

CoreLink должен быть достаточно быстрым для промышленной HMI, но читаемость не приносится в жертву микроскопической оптимизации.

Копирование небольшого snapshot, FFI-вызов раз в 500 мс и декодирование десятков значений не являются основанием для сложной инфраструктуры без измеренной проблемы.

## 49. Тестируемость

Тестируется реальная логика:

- validation;
- conversion words → values;
- word order;
- poll-plan generation;
- ID lookup;
- write encoding;
- state transitions;
- обработка ошибок.

Не создавать большую mock-инфраструктуру только ради формального coverage.

## 50. Fail Fast

Ошибки, делающие запуск невозможным, обнаруживаются до `Running`: некорректная конфигурация, неподдерживаемый тип, некорректная карта, ошибка безопасности.

Потеря PLC после успешного запуска не является Fail Fast ситуацией.

---

## 51. Что означает универсальность

Универсальность CoreLink означает, что одно Rust-ядро работает с разными Modbus-картами, которые .NET передаёт при запуске.

```text
Nitrogen HMI
↓
configuration
↓
CoreLink
```

```text
Pump HMI
↓
different configuration
↓
same CoreLink
```

Универсальность не означает поддержку всех промышленных протоколов.

## 52. Что CoreLink не должен знать

CoreLink не содержит предметных понятий вроде `Pump1Rpm`, `Pressure`, `Temperature`, `SCFM`, `Stage`, `HT400`, `Nitrogen`, `Cement`.

Для CoreLink существуют:

```text
Parameter ID
Register Area
Address
Data Type
Word Order
Value
```

Предметный смысл принадлежит приложению и PLC.

---

## 53. Общая архитектура

```text
             Avalonia / .NET 10
                     │
         configuration / writes
           get latest snapshot
                     │
                 C ABI / FFI
                     │
                     ▼
             ┌───────────────┐
             │   CoreLink    │
             │ Configuration │
             │ Security      │
             │ Latest State  │
             └───────┬───────┘
                     │
              Modbus Worker
                     │
           ┌─────────┴─────────┐
           │                   │
      Write Priority        Poller
           │                   │
           └─────────┬─────────┘
                     │
                 Modbus TCP
                     │
                     ▼
                    PLC
```

## 54. Полная последовательность запуска

```text
.NET starts
↓
core_create
↓
pass connection configuration
↓
pass parameter definitions
↓
core_configure
↓
Rust validates configuration
↓
core_start
↓
HWID
↓
Ed25519 verification
↓
┌───────────────┐
│               │
FAILED        SUCCESS
│               │
Locked          ↓
          Create Modbus runtime
                ↓
           Start worker
                ↓
           Start poller
                ↓
             Running
```

## 55. Рабочий цикл

```text
CoreLink
   │
Poll due
   │
pending Write?
 ┌─┴─┐
YES  NO
 │    │
Write Poll
 │    │
 └─┬──┘
   │
Modbus
   │
  PLC
   │
decode
   │
LatestSnapshot
   │
sequence
```

## 56. Цикл .NET

```text
Rx interval 500 ms
↓
CoreLink.GetLatest(lastSequence)
↓
┌───────────┐
│           │
NewData   NoData
│           │
Snapshot  nothing new
│
Rx publication
↓
ViewModel
↓
View
```

## 57. Цикл записи

```text
View
↓
ViewModel
↓
CoreLink.Write(ID, Value)
↓
Write Queue
↓
current transaction completes
↓
Write has priority
↓
Modbus Write
↓
WriteResult
↓
Poll
↓
PLC actual state
↓
LatestSnapshot
↓
Rx
↓
UI
```

## 58. Ошибка связи

```text
Poll / Write
↓
Timeout / Transport / Protocol Error
↓
Invalidate connection
↓
Disconnected
↓
Latest snapshot remains available as stale
↓
Reconnect policy
↓
Connect
↓
Successful Poll
↓
New Snapshot
↓
Connected / Good
```

---

## 59. Основные запреты

Без отдельного обоснованного требования запрещаются:

- `panic!` в production-path;
- `unwrap()` в production-path;
- `expect()` в production-path;
- JSON-карта регистров;
- строки параметров через FFI;
- несколько владельцев Modbus socket;
- параллельные Read/Write на одном соединении;
- второе соединение ради Write;
- бесконечные очереди;
- накопление истории snapshots;
- runtime-изменение конфигурации;
- mutable global state;
- detached tasks;
- unsafe вне минимально необходимой области;
- callbacks Rust → .NET на первом этапе;
- gRPC;
- REST;
- MQTT;
- OPC UA;
- Plugin System;
- ProtocolFactory;
- Repository pattern;
- дополнительные архитектурные слои без ответственности;
- файлы и каталоги «на будущее»;
- traits без необходимости;
- factories без необходимости;
- premature optimization;
- встроенная система логирования на первом этапе.

## 60. Критерий создания нового компонента

Перед созданием crate, module, struct, enum, trait, service, worker, queue или abstraction задаётся вопрос:

> Какую конкретную проблему текущего CoreLink решает этот компонент?

Допустимые причины:

- устраняет реальное дублирование;
- отделяет действительно другую ответственность;
- необходим для FFI;
- необходим для безопасности;
- необходим для Modbus;
- необходим для concurrency/lifecycle;
- существенно повышает читаемость.

Недопустимые причины:

- «может пригодиться позже»;
- «так обычно делают»;
- «так будет более enterprise»;
- «вдруг потом добавим другой протокол».

В этих случаях компонент не создаётся.

## 61. Критерий качества архитектуры

Разработчик должен быстро отвечать на вопросы:

- Где проверяется безопасность?
- Кто владеет Modbus socket?
- Кто выполняет polling?
- Что происходит при timeout?
- Где выполняется reconnect?
- Как Write получает приоритет?
- Где хранится последнее состояние?
- Как ID превращается в Modbus address?
- Кто преобразует два Word в Float?
- Как CoreLink останавливается?

Если для ответа приходится проходить через множество interfaces, factories, providers и слоёв — архитектура стала слишком сложной.

## 62. Первый этап реализации

Реализовать только:

1. создание CoreLink;
2. FFI boundary;
3. получение connection configuration;
4. получение parameter definitions;
5. validation;
6. HWID;
7. Ed25519 verification;
8. один Modbus worker;
9. внутренний Poller;
10. формирование Poll Plan;
11. чтение Modbus;
12. преобразование значений;
13. LatestSnapshot;
14. Sequence;
15. получение snapshot через FFI;
16. bounded Write queue;
17. приоритет Write;
18. запись Modbus;
19. timeout;
20. reconnect;
21. stop;
22. destroy.

После этого архитектуру остановить и оценить. Следующий инфраструктурный уровень автоматически не добавлять.

---

## 63. Основной принцип CoreLink

CoreLink является защищённым владельцем коммуникации с PLC.

.NET один раз описывает ему конкретную Modbus-конфигурацию. CoreLink проверяет конфигурацию и безопасность. Только после успешной авторизации запускается коммуникационный контур.

Rust самостоятельно:

```text
connects
polls
decodes
writes
handles timeout
reconnects
stores latest state
```

.NET:

```text
configures
sends commands
reads latest state
displays result
```

PLC остаётся источником технологической истины.

## 64. Финальное правило

При выборе между более универсальным, более абстрактным и более сложным решением и простым, читаемым решением, полностью закрывающим текущую задачу, CoreLink выбирает второе.

Архитектура должна оставаться настолько простой, насколько позволяет реальная задача, но не проще требований к безопасности, надёжности и детерминированности промышленной HMI.
