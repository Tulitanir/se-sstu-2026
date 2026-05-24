# 5.2. Внешние Интерфейсы

## 1. HTTP часть API для Procoting Server

Предназначен для работы с поступающими HTTP-запросами в сервер Прокторинга для работы с медиа и событиями прокторинга и античита

*Страница с swagger (если не работает iframe ниже): <https://tulitanir.github.io/se-sstu-2026/swagger/proctoring-api.html>*

<iframe src="../swagger/proctoring-api.html" width="100%" height="1000px" style="border:0;" allowfullscreen="allowfullscreen"></iframe> 

### Описание маршрутов

| Endpoint | PUT /media-sessions/{session_id} |
| :--- | :--- |
| Назначение | Получить детальную информацию о конкретном медиапотоке |
| Входные поля | `session_id` (в пути) |
| Выходные поля | `MediaSession`: `session_id`, `student_id`, `exam_id`, `status`, `started_at`, `ended_at`, `stream_url`, `etag` |
| Ошибки | `400`, `401`, `403`, `404`, `502` |
| Версия | `v1` |
| Формат | JSON |
| Обеспечение идемпотентности | Не требуется (GET-метод не изменяет состояние) |

---

| Endpoint | PUT /media-sessions/{session_id} |
| :--- | :--- |
| Назначение | Обновить статус существующего медиапотока |
| Входные поля | `session_id` (в пути), `status`, `reason` |
| Выходные поля | Обновлённый `MediaSession` |
| Ошибки | `400`, `401`, `403`, `404`, `409`, `412`, `502` |
| Версия | `v1` |
| Формат | JSON |
| Обеспечение идемпотентности** | Клиент передаёт заголовок `If-Match: <ETag>` для предотвращения конфликтов |

---

| Endpoint | DELETE /media-sessions/{session_id} |
| :--- | :--- |
| Назначение | Завершить медиапоток |
| Входные поля | `session_id` (в пути) |
| Выходные поля |- (статус `204 No Content`) |
| Ошибки | `400`, `401`, `403`, `404`, `502` |
| Версия | `v1` |
| Формат | JSON (для ошибок) |
| Обеспечение идемпотентности | Не требуется (DELETE идемпотентен по определению) |

---

| Endpoint | POST /anticheat/events |
| :--- | :--- |
| Назначение | Отправить событие античита |
| Входные поля | `event_id`, `session_id`, `event_type`, `event_data_json`, `severity`, `detected_at_unix_ms`, `offset_from_start_ms` |
| Выходные поля** | - (статус `200` без тела, только заголовки) |
| Ошибки | `400`, `401`, `403`, `404`, `429`, `502` |
| Версия | `v1` |
| Формат | JSON |
| Обеспечение идемпотентности | Передать заголовок `Idempotency-Key: <UUID>`. Ответ может содержать `X-Idempotent-Replayed: true` |

---

| Endpoint | POST /processed/events |
| :--- | :--- |
| Назначение | Отправить решение проктора по событию |
| Входные поля | `event_id`, `session_id`, `decision` : `decision_id`, `proctor_id`, `type`, `comment`, `decided_at_unix_ms` |
| Выходные поля | - (статус `200` без тела, только заголовки) |
| Ошибки | `400`, `401`, `403`, `404`, `409`, `502` |
| Версия | `v1` |
| Формат | JSON |
| Обеспечение идемпотентности | Передать заголовок `Idempotency-Key: <UUID>`. Ответ может содержать `X-Idempotent-Replayed: true` |

---

## 2. WebSockets часть API для Procoting Server

Предназначен преимущественно для событий для системы прокторинга. API позволяет клиентам подписываться на события античита и получать обработанные события прокторинга в реальном времени.

### Основной маршрут: `/v1/ws`

### Каналы событий

#### 1. Канал событий античита
- Название: `anticheat.events`
- Описание: Канал для получения событий, связанных с подозрительной активностью студента во время экзамена.
- JSON-cхема сообщения:
```
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://example.com/schemas/anticheat-event-v1.json",
  "title": "AnticheatEvent",
  "description": "Событие античита, фиксирующее нарушения и подозрительные действия пользователя",
  "type": "object",
  "properties": {
    "event_id": {
      "type": "string",
      "description": "Уникальный идентификатор события"
    },
    "session_id": {
      "type": "string",
      "description": "Идентификатор экзаменационной сессии"
    },
    "event_type": {
      "type": "integer",
      "description": "Тип события",
      "enum": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 99],
      "enumNames": [
        "EVENT_TYPE_UNSPECIFIED",
        "TAB_SWITCH",
        "WINDOW_BLUR",
        "SCREEN_CAPTURE_DETECTED",
        "VM_DETECTED",
        "DEBUGGER_DETECTED",
        "RAPID_TYPING",
        "IDLE_TIMEOUT",
        "CONNECTION_LOST",
        "CONNECTION_RESTORED",
        "HIGH_LATENCY",
        "SUSPICIOUS_BEHAVIOR",
        "VIOLATION_CONFIRMED",
        "CUSTOM"
      ]
    },
    "severity": {
      "type": "integer",
      "description": "Уровень серьёзности",
      "enum": [0, 1, 2, 3, 4],
      "enumNames": [
        "SEVERITY_UNSPECIFIED",
        "INFO",
        "WARNING",
        "SUSPICIOUS",
        "CRITICAL"
      ]
    },
    "detected_at_unix_ms": {
      "type": "integer",
      "description": "Время фиксации события в миллисекундах (Unix)",
      "minimum": 0
    },
    "offset_from_start_ms": {
      "type": "integer",
      "description": "Время относительно начала экзамена в миллисекундах",
      "minimum": 0
    },
    "tab_switch": {
      "type": "object",
      "description": "Детали переключения вкладки",
      "properties": {
        "switch_count": {
          "type": "integer",
          "description": "Счётчик переключений за сессию",
          "minimum": 0
        },
        "duration_ms": {
          "type": "integer",
          "description": "Длительность отсутствия фокуса в миллисекундах",
          "minimum": 0
        },
        "switched_to_app": {
          "type": "string",
          "description": "Название приложения/вкладки"
        }
      },
      "additionalProperties": false
    },
    "app_switch": {
      "type": "object",
      "description": "Детали обнаружения приложений",
      "properties": {
        "app_name": {
          "type": "string",
          "description": "Название приложения"
        },
        "app_bundle_id": {
          "type": "string",
          "description": "Идентификатор приложения"
        },
        "is_blocked": {
          "type": "boolean",
          "description": "Находится ли в чёрном списке"
        },
        "duration_ms": {
          "type": "integer",
          "description": "Длительность работы приложения в миллисекундах",
          "minimum": 0
        }
      },
      "additionalProperties": false
    },
    "system_violation": {
      "type": "object",
      "description": "Системные нарушения",
      "properties": {
        "violation_type": {
          "type": "string",
          "description": "Тип подозрения/нарушения: Виртуальная машина, захват экрана и прочее"
        },
        "evidence": {
          "type": "string",
          "description": "Детали/подтверждения обнаружения"
        },
        "is_deterministic": {
          "type": "boolean",
          "description": "Подтвержденный факт или подозрение?"
        }
      },
      "additionalProperties": false
    },
    "network_issue": {
      "type": "object",
      "description": "Сетевые проблемы",
      "properties": {
        "lost_packets_percent": {
          "type": "integer",
          "description": "Процент потерянных пакетов",
          "minimum": 0,
          "maximum": 100
        },
        "latency_ms": {
          "type": "integer",
          "description": "Текущая задержка в миллисекундах",
          "minimum": 0
        },
        "reconnects_count": {
          "type": "integer",
          "description": "Количество переподключений",
          "minimum": 0
        }
      },
      "additionalProperties": false
    },
    "custom_event": {
      "type": "object",
      "description": "Кастомное событие",
      "properties": {
        "custom_type": {
          "type": "string",
          "description": "Произвольный тип события"
        },
        "payload_json": {
          "type": "string",
          "description": "JSON с произвольными данными"
        }
      },
      "additionalProperties": false
    },
    "event_hash": {
      "type": "string",
      "description": "Контрольная сумма события"
    }
  },
  "required": ["event_id", "session_id", "event_type", "severity", "detected_at_unix_ms", "event_hash"],
  "allOf": [
    {
      "oneOf": [
        { "required": ["tab_switch"] },
        { "required": ["app_switch"] },
        { "required": ["system_violation"] },
        { "required": ["network_issue"] },
        { "required": ["custom_event"] }
      ]
    }
  ],
  "additionalProperties": false
}
```

### 2. Канал обработанных событий анализа медиапотоков
- Название: `processed.events`
- Описание: Канал для получения событий анализа медиапотоков после их обработки проктором.
- JSON-cхема сообщения:
```
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://example.com/schemas/processed-proctoring-event-v1.json",
  "title": "ProcessedStreamEvent",
  "description": "Обработанное событие, зафиксированное и подтверждённое проктором",
  "type": "object",
  "properties": {
    "event_id": {
      "type": "string",
      "description": "Уникальный идентификатор события"
    },
    "session_id": {
      "type": "string",
      "description": "Идентификатор экзаменационной сессии"
    },
    "decision": {
      "type": "object",
      "description": "Решение проктора",
      "properties": {
        "decision_id": {
          "type": "string",
          "description": "Уникальный ID решения"
        },
        "proctor_id": {
          "type": "string",
          "description": "ID проктора, принявшего решение"
        },
        "type": {
          "type": "integer",
          "description": "Тип решения",
          "enum": [0, 1, 2, 3, 4, 5],
          "enumNames": [
            "DECISION_UNSPECIFIED",
            "WARNING",
            "VIOLATION_CONFIRM",
            "VIOLATION_REJECT",
            "EXAM_TERMINATED",
            "REVIEW_REQUIRED"
          ]
        },
        "comment": {
          "type": "string",
          "description": "Комментарий проктора"
        },
        "decided_at_unix_ms": {
          "type": "integer",
          "description": "Время принятия решения в миллисекундах (Unix)",
          "minimum": 0
        }
      },
      "required": ["decision_id", "proctor_id", "type", "decided_at_unix_ms"],
      "additionalProperties": false
    },
    "violation": {
      "type": "object",
      "description": "Данные о нарушении",
      "properties": {
        "original_event_id": {
          "type": "string",
          "description": "ID исходного события"
        },
        "type": {
          "type": "integer",
          "description": "Тип нарушения",
          "enum": [0, 1, 2, 3, 4],
          "enumNames": [
            "VIOLATION_UNSPECIFIED",
            "TAB_SWITCH_EXCESS",
            "MULTIPLE_FACES",
            "NO_FACE_LONG",
            "GAZE_ANGLE_EXCESS"
          ]
        },
        "description": {
          "type": "string",
          "description": "Описание нарушения"
        },
        "detected_at_unix_ms": {
          "type": "integer",
          "description": "Когда произошло нарушение в миллисекундах (Unix)",
          "minimum": 0
        },
        "severity_score": {
          "type": "integer",
          "description": "Степень серьёзности",
          "minimum": 0,
          "maximum": 100
        },
        "is_automatic": {
          "type": "boolean",
          "description": "Автоматическое обнаружение или ручное"
        }
      },
      "required": ["original_event_id", "type", "detected_at_unix_ms"],
      "additionalProperties": false
    },
    "created_at_unix_ms": {
      "type": "integer",
      "description": "Время создания события на сервере в миллисекундах (Unix)",
      "minimum": 0
    },
    "expires_at_unix_ms": {
      "type": "integer",
      "description": "Время, когда уведомление теряет актуальность, в миллисекундах (Unix)",
      "minimum": 0
    },
    "event_hash": {
      "type": "string",
      "description": "Контрольная сумма события"
    }
  },
  "required": ["event_id", "session_id", "decision", "violation", "created_at_unix_ms", "expires_at_unix_ms", "event_hash"],
  "additionalProperties": false
}
```