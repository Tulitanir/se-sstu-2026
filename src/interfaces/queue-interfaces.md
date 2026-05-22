# 5.1. Внутренние Интерфейсы

Представлены информация о событии, типе взаимодействия и схемы для очередей в проектируемой системе:
1. Settings
2. Response
3. Anticheat Events
4. Proccessed Proctoring Events

## 1. Settings
Очередь settings предназначена для перессылки сообщений об изменении настроек прокторинга и чувстввительности антчита от основного сервера до сервера прокторинга

### Общая информация о событии/эндпоинте

|Параметр            |	Значение                                   |
|--------------------|---------------------------------------------|
|Тип взаимодействия  |очередь (Kafka)                              |
|Название топика     |proctoring.settings.v1                       |
|Producer            |Основной Backend (Go, Policy Manager)        |
|Consumer            |Сервер Прокторинга (Python, Settings Handler)|
|Гарантия доставки   |at-least-once                                |
|Требования Retry    |3 попытки с экспоненциальной задержкой       |

### Protobuf-схема с описанием полей
```
syntax = "proto3";

package proctoring.settings.v1;

option go_package = "proctoring/settings/v1;settingsv1";

// Сообщение для публикации изменения настроек прокторинга
message ProctoringSettingsUpdate {
  // Уникальный идентификатор (UUID)
  string policy_version_id = 1;
  
  // Тип операции
  OperationType operation = 2;
  
  // Временная метка изменения (unix timestamp, в миллисекундах)
  int64 changed_at_ms = 3;
  
  // Идентификатор пользователя, применившего изменение
  string changed_by_user_id = 4;
  
  // Набор правил прокторинга
  ProctoringRules rules = 5;
  
  // Информация для валидации и целостности
  ValidationInfo validation = 6;
}

enum OperationType {
  OPERATION_UNSPECIFIED = 0;
  DRAFT_UPDATED = 1;       // изменение черновика - не применяется к новым экзаменам
  VERSION_PUBLISHED = 2;   // публикация новой версии - применяяется к новым экзаменам
  VERSION_ROLLBACK = 3;    // возврат к предыдущей версии
}

// Основные настройки прокторинга
message ProctoringRules {
  
  // Параметры античита
  AnticheatConfig anticheat = 1;

  // Параметры анализатора медиапотока
  StreamConfig streamconfig = 2;
}

message StreamConfig {
    // Чувствительность детектора лиц (0.0 - 1.0)
    float face_detection_sensitivity = 1;

    // Чувствительность детектора на второе лицо в кадре (0.0 - 1.0)
    float multi_face_sensitivity = 2;
    
    // Чувствительность детектора на отсутствие лица (0.0 - 1.0)
    float no_face_sensitivity = 3;
    
    // Разрешённое отклонение взгляда в градусах
    int32 gaze_angle_threshold_deg = 4;
    
    // Включена ли проверка посторонних приложений
    bool check_unauthorized_apps = 5;
}

// Конфигурация античита
message AnticheatConfig {
  // Максимальное допустимое количество переключений вкладок
  int32 max_tab_switches = 1;
  
  // Максимальное количество подозрительных событий до фиксации нарушения
  int32 max_suspicious_events = 2;

  // Включённые типы проверок в виде битоввой маски
  int32 enabled_checks_mask = 3;
}

// Информация для валидации и верификации
message ValidationInfo {
  // Контрольная сумма настроек (SHA-256)
  string settings_hash = 1;
  
  // Обязательность применения: true - должна быть применена обязательно
  bool mandatory = 2;
  
  // Ожидаемое время применения, в миллисекнудах
  int64 expected_apply_deadline_ms = 3;
}
```

### Возможные ошибки при обработке
1. VALIDATION_HASH_MISMATCH	- settings_hash не соответствует вычисленному
2. POLICY_VERSION_ALREADY_APPLIED - версия уже применена, дубликат policy_version_id
3. APPLY_TIMEOUT - Превышено ожидаемое время применения (> expected_apply_deadline_ms)

---

## 2. Response
Очередь response предназначена для получения подтверждений об изменении настроек прокторинга со стороны сервера прокторинга

### Общая информация о событии/эндпоинте

|Параметр            |	Значение                                   |
|--------------------|---------------------------------------------|
|Тип взаимодействия  |очередь (Kafka)                              |
|Название топика     |proctoring.settings.response.v1              |
|Producer            |Сервер Прокторинга (Python, Settings Handler)|
|Consumer            |Основной Backend (Go, Policy Manager)        |
|Гарантия доставки   |at-least-once                                |

### Protobuf-схема с описанием полей
```
syntax = "proto3";

package proctoring.settings.response.v1;

option go_package = "proctoring/settings/response/v1;responsev1";

// Cообщение для подтверждения применения настроек
message SettingsApplyConfirmation {
    // Уникальный идентификатор (из исходного запроса)
    string policy_version_id = 1;
    
    // Результат применения
    ApplyResult result = 2;
    
    // Временная метка, когда сервер прокторинга завершил попытку
    int64 processed_at = 3;
    
    // Идентификатор экземпляра сервера прокторинга
    string proctoring_instance_id = 4;
    
    // Детали результата 
    oneof details {
        SuccessDetails success = 5;
        FailureDetails failure = 6;
        PartialSuccessDetails partial_success = 7;
    }
}

// Успешное применение
message SuccessDetails {
    // Время, когда настройки вступили в силу
    int64 applied_at = 1;
    
    // Идентификатор версии, которая была активна до этого (для откатов)
    string previous_policy_version_id = 2;
    
    // Контрольная сумма применённых настроек
    string applied_settings_hash = 3;
}

// Полная неудача при применении
message FailureDetails {
    // Код ошибки
    FailureCode code = 1;
    
    // Описание
    string message = 2;
    
    // Была ли сделана повторная попытка
    bool retry_attempted = 3;
}

// Частичное применение (например, античит обновился, а прокторинг медиапотока - нет)
message PartialSuccessDetails {
    // Какие компоненты успешно обновлены
    repeated Component updated_components = 1;
    
    // Какие компоненты не обновлены
    repeated Component failed_components = 2;
    
    // Общее сообщение о статусе
    string summary = 3;
    
    // Таймаут, через который будет повторная попытка (ms)
    int32 retry_after_ms = 4;
}

// Результат применения (enum)
enum ApplyResult {
    RESULT_UNSPECIFIED = 0;
    SUCCESS = 1;              // полностью успешно
    FAILURE = 2;              // полностью не успешно
    PARTIAL_SUCCESS = 3;      // частично успешно
}

// Коды ошибок
enum FailureCode {
    FAILURE_UNSPECIFIED = 0;
    VALIDATION_HASH_MISMATCH = 1;        // хеш не совпал 
    POLICY_VERSION_ALREADY_APPLIED = 2;  // версия уже применена
    APPLY_TIMEOUT = 3;                   // превышен лимит ожидания применения
    INTERNAL_ERROR = 4;                  // прочая внутренняя ошибка
}

// Компоненты системы прокторинга
enum Component {
    COMPONENT_UNSPECIFIED = 0;
    ANTICHEAT_HANDLER = 1;
    STREAM_HANDLER = 2;
}
```

## 3. Anticheat Events
Очередь anticheat предназначена для накопления событий, зафиксированных античит-модулей на основе событий действий пользователя в системе

### Общая информация о событии/эндпоинте

|Параметр            |	Значение                                    |
|--------------------|----------------------------------------------|
|Тип взаимодействия  |очередь (Kafka)                               |
|Название топика     |proctoring.anticheat.events.v1                |
|Producer            |Anticheat Handler (Python)                    |
|Consumer            |1. Archive Manager (Python)<br>2. Proctoring API (Python) |
|Гарантия доставки   |at-least-once                                 |

### Protobuf-схема с описанием полей
```
syntax = "proto3";

package proctoring.anticheat.events.v1;

option go_package = "proctoring/anticheat/events/v1;eventsv1";

// Cообщение для событий античита
message AnticheatEvent {
    // Уникальный идентификатор события
    string event_id = 1;
    
    // Идентификатор экзаменационной сессии (связь с прокторинг-сессией)
    string session_id = 2;
    
    // Тип события
    EventType event_type = 3;
    
    // Уровень серьёзности
    SeverityLevel severity = 4;
    
    // Время фиксации события
    int64 detected_at_unix_ms = 5;
    
    // Время относительно начала экзамена
    int64 offset_from_start_ms = 6;
    
    // Метаданные события
    oneof details {
        TabSwitchDetails tab_switch = 8;
        AppSwitchDetails app_switch = 9;
        SystemViolationDetails system_violation = 10;
        NetworkIssueDetails network_issue = 11;
        CustomEventDetails custom_event = 12;
    }
    
    // Контрольная сумма события
    string event_hash = 15;
}

// Типы событий античита
enum EventType {
    EVENT_TYPE_UNSPECIFIED = 0;
    
    // Нарушения переключения контекста
    TAB_SWITCH = 1;              // Переключение вкладки
    WINDOW_BLUR = 2;             // Потеря фокуса окна

    // Приложения и окружение
    SCREEN_CAPTURE_DETECTED = 3;     // Обнаружена попытка захвата экрана
    VM_DETECTED = 4;                 // Обнаружена виртуальная машина
    DEBUGGER_DETECTED = 5;           // Обнаружен отладчик
    
    // Поведенческие аномалии
    RAPID_TYPING = 6;                // Аномально быстрый ввод
    IDLE_TIMEOUT = 7;                // Долгое бездействие
    
    // Сетевые и системные
    CONNECTION_LOST = 8;             // Потеря соединения
    CONNECTION_RESTORED = 9;         // Восстановление соединения
    HIGH_LATENCY = 10;                // Высокая задержка (> 500 мс)
    
    // Составные нарушения
    SUSPICIOUS_BEHAVIOR = 19;         // Комплексное подозрительное поведение
    VIOLATION_CONFIRMED = 20;         // Нарушение подтверждено проктором
    
    // Расширяемые типы (для других сценариев)
    CUSTOM = 99;
}

// Уровень серьёзности
enum SeverityLevel {
    SEVERITY_UNSPECIFIED = 0;
    INFO = 1;        // Информационное событие
    WARNING = 2;     // Подозрение
    SUSPICIOUS = 3;  // Требует внимания проктора
    CRITICAL = 4;    // Явное нарушение
}

// Детали переключения вкладки
message TabSwitchDetails {
    int32 switch_count = 1;           // Счётчик переключений за сессию
    int32 duration_ms = 2;            // Длительность отсутствия фокуса
    string switched_to_app = 3;       // Название приложения/вкладки
}

// Детали обнаружения приложений
message AppSwitchDetails {
    string app_name = 1;               // Название приложения
    string app_bundle_id = 2;          // Идентификатор приложения
    bool is_blocked = 3;               // Находится ли в чёрном списке
    int32 duration_ms = 4;             // Длительность работы приложения
}

// Системные нарушения
message SystemViolationDetails {
    string violation_type = 1;         // Виртуальная машина, захват экрана и прочее
    string evidence = 2;               // Детали обнаружения
    bool is_deterministic = 3;         // Это подтвержденный факт или подозрение
}

// Сетевые проблемы
message NetworkIssueDetails {
    int32 lost_packets_percent = 1;    // Процент потерянных пакетов
    int32 latency_ms = 2;              // Текущая задержка
    int32 reconnects_count = 3;        // Количество переподключений
}

// Кастомное событие (для расширения фиксируемых событий)
message CustomEventDetails {
    string custom_type = 1;             // Произвольный тип события
    string payload_json = 2;            // JSON с произвольными данными
}
```

## 4. Proccessed Stream Event
Очередь Proccessed Proctoring Event предназначена для накопления событий прокторинга, зафиксированных и подтвержденных проктором

### Общая информация о событии/эндпоинте

|Параметр            |	Значение                                    |
|--------------------|----------------------------------------------|
|Тип взаимодействия  |очередь (Kafka)                               |
|Название топика     |stream.processed.events.v1                |
|Producer            |Proctoring API (Python)                       |
|Consumer            |Archive Manager (Python)                      |
|Гарантия доставки   |at-least-once                                 |

### Protobuf-схема с описанием полей
```
syntax = "proto3";

package stream.processed.events.v1;

option go_package = "stream/processed/events/v1;eventsv1";

// Корневое сообщение для обработанных событий
message ProcessedStreamEvent {
    // Уникальный идентификатор события
    string event_id = 1;
    
    // Идентификатор экзаменационной сессии
    string session_id = 2;
    
    // Решение проктора
    ProctorDecision decision = 3;
    
    // Данные о нарушении
    ViolationInfo violation = 4;
    
    // Временные метки
    int64 created_at_unix_ms = 5;      // когда создано на сервере
    int64 expires_at_unix_ms = 6;      // когда уведомление теряет актуальность
    
    // Контрольная сумма
    string event_hash = 9;
}

// Решение проктора
message ProctorDecision {
    string decision_id = 1;             // Уникальный ID решения
    string proctor_id = 2;              // ID проктора, принявшего решение
    DecisionType type = 3;              // Тип решения
    string comment = 4;                 // Комментарий проктора
    int64 decided_at_unix_ms = 5;       // Время принятия решения
}

enum DecisionType {
    DECISION_UNSPECIFIED = 0;
    WARNING = 1;                       // Вынести предупреждение
    VIOLATION_CONFIRMED = 2;           // Подтвердить нарушение
    VIOLATION_REJECTED = 3;            // Отклонить нарушение
    EXAM_TERMINATED = 4;                // Прервать экзамен
    REVIEW_REQUIRED = 5;               // Требуется ручная проверка
}

// Информация о нарушении
message ViolationInfo {
    string original_event_id = 1;       // ID события
    ViolationType type = 2;             // Тип нарушения
    string description = 3;             // Описание
    int64 detected_at_unix_ms = 4;      // Когда произошло
    int32 severity_score = 5;           // Степень серьезности
    bool is_automatic = 6;              // Автоматическое обнаружение или ручное
}

enum ViolationType {
    VIOLATION_UNSPECIFIED = 0;
    TAB_SWITCH_EXCESS = 1;              // Превышение переключений вкладок
    MULTIPLE_FACES = 2;                 // Несколько лиц в кадре
    NO_FACE_LONG = 3;                   // Долгое отсутствие лица
    GAZE_ANGLE_EXCESS = 4;              // Частое отведение взгляда
}
```
