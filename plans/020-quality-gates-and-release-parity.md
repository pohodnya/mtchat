# Plan 020: Quality Gates and Release Parity

**Status:** Implemented
**Date:** 2026-05-05

## Summary

Усилить инженерный контур проекта: CI, release workflow и локальные команды проверки должны покрывать одинаковые критические риски для backend и frontend.

## Problem

Сейчас quality gates асимметричны:

- backend проверяется заметно строже, чем frontend
- frontend tests существуют, но не входят в CI/release как обязательный gate
- release workflow не повторяет все важные проверки из обычного CI
- `@mtchat/vue-primevue` участвует в CI, но слабо представлен в release validation

Из-за этого релиз может пройти при деградации JS-части или при расхождении release и CI контуров.

## Goals

- Один понятный набор обязательных quality gates
- Release validation не слабее обычного CI
- Frontend packages проверяются не только сборкой, но и тестами
- Локальный contributor понимает, какой командой проверить репозиторий перед PR

## Non-Goals

- Миграция на monorepo toolchain
- Введение e2e-инфраструктуры в этой итерации
- Полный security pipeline с SAST/DAST

## Scope

### 1. Frontend CI hardening

- Добавить `npm test` для `mtchat-vue` в CI
- Определить минимальный тестовый контур для `mtchat-vue-primevue`
- При необходимости добавить smoke-tests для wrapper package

### 2. Release parity

- Выровнять `release.yml` с обязательными проверками из `ci.yml`
- Добавить недостающие сервисы/env для integration tests
- Убедиться, что release не публикует артефакты без полного validate

### 3. Local developer workflow

- Сформировать единый pre-PR checklist
- По возможности добавить root-level documented verification commands
- Упростить понимание порядка команд для backend/frontend maintainers

## Deliverables

- Обновлённый `.github/workflows/ci.yml`
- Обновлённый `.github/workflows/release.yml`
- При необходимости test script для `mtchat-vue-primevue`
- Документация по локальной верификации перед PR/release

## Implementation

### Phase 1. Gate Matrix

- Составить матрицу текущих проверок:
  - backend lint/build/tests
  - frontend typecheck/build/tests
  - packaging checks
  - release-only steps
- Отметить пробелы между CI и release

### Phase 2. CI Strengthening

- Включить frontend unit tests в обязательный pipeline
- Добавить wrapper package checks, если сейчас они недостаточны
- Проверить время выполнения и кэширование

### Phase 3. Release Parity

- Перенести обязательные проверки из CI в release validate
- Добавить Redis туда, где он нужен integration tests
- Убедиться, что validate покрывает оба npm packages

### Phase 4. Documentation

- Описать команды локальной верификации
- Зафиксировать минимальный набор проверок перед tag/release

## Task Checklist

- [x] Описать текущую матрицу quality gates
- [x] Добавить `mtchat-vue` tests в CI
- [x] Решить стратегию тестов для `mtchat-vue-primevue`
- [x] Добавить недостающие frontend checks в release validate
- [x] Выровнять backend integration env между CI и release
- [x] Проверить порядок зависимостей jobs и reuse артефактов
- [x] Описать локальный pre-PR/pre-release checklist

## Validation

- Локальный прогон ключевых команд:
  - `cargo test --manifest-path mtchat-rust/Cargo.toml --lib`
  - `cargo test --manifest-path mtchat-rust/Cargo.toml --tests`
  - `npm test` в `mtchat-vue`
  - `npm run build` в `mtchat-vue`
  - `npm run build` в `mtchat-vue-primevue`
- Review workflow diff на предмет parity CI vs Release

## Risks

- Более строгий pipeline увеличит время CI
- Frontend tests могут оказаться flaky и потребуют предварительной стабилизации
- Попытка быстро дотянуть release parity может вскрыть скрытые зависимости на локальное окружение

## Done Criteria

- Frontend unit tests входят в обязательный CI gate
- Release validate не слабее обычного CI по критическим проверкам
- Оба npm packages явно присутствуют в release validation
- Есть документированный локальный способ проверить проект перед PR и перед release
