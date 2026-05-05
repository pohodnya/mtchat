# Plan 021: Repository Hygiene and Test Stability

**Status:** Implemented
**Date:** 2026-05-05

## Summary

Очистить репозиторий от эксплуатационного шума и стабилизировать тестовую среду, чтобы кодовая база оставалась reviewable, воспроизводимой и удобной для контрибьюции.

## Problem

В проекте уже есть зрелые инфраструктурные элементы, но hygiene проседает:

- в рабочем дереве накапливаются `node_modules`, `dist`, `target`, `site`
- в дереве встречаются `.DS_Store` и другие локальные артефакты
- frontend tests проходят с лишним шумом и не полностью изолированы от сетевых попыток
- отсутствует формализованный подход к clean workspace перед review/release

Это не ломает продукт напрямую, но снижает инженерную дисциплину и мешает развитию.

## Goals

- Репозиторий и рабочее дерево чище и предсказуемее
- Тесты не делают случайных сетевых запросов и не шумят без причины
- Контрибьютор понимает, какие артефакты допустимы, а какие нет
- Проверки hygiene автоматизируются настолько, насколько это практично

## Non-Goals

- Историческая перепаковка git history
- Полное удаление всех build outputs из локальной разработки
- Переезд на другой тестовый раннер

## Scope

### 1. Workspace hygiene

- Зафиксировать политику по generated artifacts
- Убедиться, что `.gitignore` покрывает реальные артефакты проекта
- По возможности добавить простую проверку на мусорные файлы

### 2. Test stability

- Найти источник `happy-dom` warning/async task noise
- Убрать сетевые side effects из sanitize/unit tests
- При необходимости замокать `fetch`, `window.open`, navigation-related APIs

### 3. Contributor ergonomics

- Описать clean-up workflow для локальной работы
- Добавить рекомендации для reviewable PRs
- Согласовать, какие generated assets допустимо коммитить, а какие нет

## Deliverables

- Обновлённые ignore/hygiene rules при необходимости
- Стабилизированные frontend unit tests без лишнего шума
- Документация по clean workspace и работе с generated artifacts
- Минимальный automation check на мусорные файлы или запрещённые артефакты

## Implementation

### Phase 1. Audit

- Перечислить реальные типы локальных артефактов
- Разделить их на:
  - допустимые publish artifacts
  - локальные build artifacts
  - мусорные OS/editor файлы

### Phase 2. Guardrails

- Уточнить `.gitignore`, если есть пробелы
- Добавить lightweight check для запрещённых файлов
- Зафиксировать политику по `dist/`, `site/`, `node_modules/`, `target/`

### Phase 3. Frontend Test Isolation

- Исправить тесты, которые вызывают нежелательные сетевые эффекты
- Замокать проблемные browser APIs
- Убедиться, что `npm test` проходит тихо и стабильно

### Phase 4. Documentation

- Описать hygiene expectations в docs/plans или contributing-oriented section
- Добавить короткий checklist перед review/release

## Task Checklist

- [x] Провести inventory generated artifacts и мусорных файлов
- [x] Проверить достаточность текущего `.gitignore`
- [x] Решить, нужны ли автоматические hygiene checks в CI
- [x] Убрать шум из `happy-dom` тестов
- [x] Убрать/замокать нежелательные сетевые обращения в unit tests
- [x] Документировать clean workspace workflow
- [x] Документировать правила коммита generated assets

## Validation

- `npm test` в `mtchat-vue` проходит без сетевого шума и лишних ошибок в stdout
- В рабочем дереве нет случайных `.DS_Store` и аналогичных артефактов после стандартной разработки
- Hygiene policy понятна без чтения истории коммитов

## Risks

- Слишком агрессивные hygiene checks могут мешать локальной разработке
- Можно случайно запретить коммит артефактов, которые реально нужны для публикации
- Шум в тестах может быть симптомом глубже лежащей проблемы в helper code или test environment

## Done Criteria

- Тестовый вывод frontend чистый и воспроизводимый
- Политика по generated artifacts формализована
- Репозиторий проще ревьюить и сопровождать
- hygiene regressions ловятся раньше, чем доходят до релиза
