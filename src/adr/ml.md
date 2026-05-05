# Выбор модели компьютерного зрения и стратегии объединения признаков для ML-подсистемы прокторинга

## Context

ML-подсистема прокторинга требует использования модели компьютерного зрения для анализа видеопотока экзаменуемого в режиме near real-time.

Необходимо решить следующие задачи:

* обнаружение лица пользователя;
* определение положения головы;
* отслеживание направления взгляда;
* определение присутствия человека в кадре;
* обнаружение второго человека;
* обнаружение запрещённых предметов (смартфон, наушники, иные устройства);
* оценка базовых поз и движений пользователя;
* агрегирование результатов разных моделей в единый риск-скор.

Дополнительные ограничения проекта:

* решение должно запускаться на стандартном серверном CPU без обязательного GPU;
* latency обработки одного кадра должна быть минимальной;
* желательно наличие готовых pretrained weights;

Рассматриваемые варианты:

1. YOLO-family detector
2. MediaPipe
3. OpenPose

---

## Decision

Принято решение использовать **комбинированный подход**:

* **MediaPipe** — для анализа лица, глаз и позы;
* **lightweight YOLO-family detector** — для обнаружения объектов и дополнительных людей;
* **OpenPose** — не использовать.

В качестве baseline implementation рассматривается актуальная lightweight-модель семейства YOLO
(например: YOLO12n / YOLO11n / аналогичная nano/small конфигурация).

Финальный выбор конкретной модели фиксируется на этапе benchmarking.

Дополнительно принято решение использовать **late feature fusion** для объединения результатов разных CV-модулей.

```text id="4v8s2r"
Frame
 ├── MediaPipe Face Mesh
 ├── MediaPipe Pose
 └── YOLO-family detector
          ↓
Feature normalization
          ↓
Late fusion
          ↓
Risk scoring
```

---

## Option 1 — YOLO-family detector

### Description

Современные модели семейства YOLO являются стандартом de-facto для real-time object detection.

Поддерживают:

* detection;
* segmentation;
* classification;
* pose estimation (в некоторых конфигурациях).

---

### Pros

* высокая скорость inference;
* хорошая точность;
* pretrained weights;
* простая интеграция;
* активное сообщество;
* удобно дообучать;
* обнаруживает несколько объектов одновременно;
* подходит для CPU/GPU deployment.

Подходит для:

* второй человек в кадре;
* телефон;
* наушники;
* посторонние предметы;
* отсутствие пользователя;
* scene analysis.

---

### Cons

Недостаточно хорошо решает:

* gaze estimation;
* micro facial landmarks;
* blink detection;
* head pose estimation без дополнительной обработки.

---

### Verdict

Использовать как **object detector**.

---

## Option 2 — MediaPipe

### Description

MediaPipe предоставляет готовые пайплайны:

* Face Detection;
* Face Mesh;
* Iris Tracking;
* Pose;
* Hands.

---

### Pros

Высокая точность для landmark detection:

* лицо;
* глаза;
* рот;
* положение головы;
* направление взгляда;
* поза.

Также:

* работает на CPU;
* очень низкий latency;
* минимальная настройка;
* простая интеграция.

---

### Cons

Не является полноценным object detector.

Плохо подходит для:

* смартфонов;
* посторонних объектов;
* пользовательских классов объектов.

---

### Verdict

Использовать как **face / pose feature extractor**.

---

## Option 3 — OpenPose

### Description

OpenPose — классический подход для human pose estimation.

---

### Pros

* высокая точность keypoints;
* хорошая детализация тела;
* зрелая технология.

---

### Cons

Для проекта имеет существенные недостатки:

* тяжёлый inference;
* высокая latency;
* CPU performance слабая;
* сложнее разворачивать;
* требует больше вычислительных ресурсов;
* избыточен для задач MVP.

---

### Verdict

Не использовать.

---

## Comparison

| Критерий            | YOLO-family |     MediaPipe |    OpenPose |
| ------------------- | ----------: | ------------: | ----------: |
| Скорость            |     высокая | очень высокая |      низкая |
| CPU friendly        |          да |            да | ограниченно |
| Face landmarks      |      средне |       отлично |         нет |
| Gaze estimation     |       слабо |       отлично |         нет |
| Pose estimation     |      хорошо |        хорошо |     отлично |
| Object detection    |     отлично |         слабо |         нет |
| Простота интеграции |     высокая |       высокая |     средняя |
| Подходит для MVP    |          да |            да |         нет |

---

## Feature Fusion Strategy

### Decision

Принято решение использовать **late fusion + weighted aggregation**.

Каждый detector независимо вычисляет собственные признаки и confidence score.

Пример:

```text id="8zj3wd"
MediaPipe:
    gaze_score = 0.72
    blink_rate = 0.21
    head_rotation = 0.34

YOLO:
    phone_prob = 0.88
    second_person_prob = 0.05

Audio:
    speech_activity = 0.41
```

После этого признаки нормализуются и объединяются в unified feature vector:

```text id="y4vpm6"
X = [
    gaze_score,
    blink_rate,
    head_rotation,
    phone_prob,
    second_person_prob,
    speech_activity
]
```

X=[x_1,x_2,x_3,x_4,x_5,x_6]

---

### Aggregation model

Итоговый риск вычисляется через weighted linear aggregation:

```text id="5m2gqk"
risk = Σ wi * xi
```

risk=\sum_{i=1}^{n} w_i x_i

где:

* `xi` — нормализованные признаки;
* `wi` — веса признаков.

Пример весов:

```text id="3r6hfk"
phone_detected      = 0.35
second_person       = 0.30
long_absence        = 0.20
gaze_anomaly        = 0.10
audio_anomaly       = 0.05
```

---

### Why not Early Fusion

Вариант:

```text id="r2j8vm"
raw features → single neural network → verdict
```

Отклонён по причинам:

* нужен большой dataset;
* сложнее обучение;
* black-box inference;
* низкая explainability;
* сложнее защита архитектурного решения.

---

### Future evolution

На следующих итерациях weighted aggregation может быть заменён на:

* XGBoost;
* LightGBM;
* CatBoost.

Это позволит реализовать **learning-based fusion**, если появится размеченный датасет.

---

## Final architecture

Итоговая схема:

```text id="k1n5pz"
Camera stream
    ↓
Frame extraction
    ↓
 ┌─────────────────────────────┐
 │ MediaPipe Face + Pose       │
 │ YOLO-family detector        │
 │ Audio analyzer              │
 └─────────────────────────────┘
    ↓
Feature Bus
    ↓
Normalization
    ↓
Late Fusion Service
    ↓
Risk Score
    ↓
Alert Queue
```

---

## Consequences

Положительные:

* высокая скорость обработки;
* хорошая точность;
* explainable scoring;
* модульность;
* простая отладка;
* подходит для учебной реализации;
* легко расширять новыми detectors.

Отрицательные:

* несколько моделей вместо одной;
* необходимость синхронизации inference pipeline;
* необходимость ручной калибровки весов на MVP.

---

## Result

Для CV-части ML-подсистемы выбран **hybrid approach**:

**MediaPipe + lightweight YOLO-family detector + late feature fusion**

Решение обеспечивает баланс между:

* качеством;
* скоростью;
* простотой реализации;
* вычислительной стоимостью;
* архитектурной расширяемостью;
* интерпретируемостью результата.
