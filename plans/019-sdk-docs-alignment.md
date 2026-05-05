# Plan 019: SDK Documentation and Package Alignment

**Status:** Implemented
**Date:** 2026-05-05

## Summary

Привести публичный контур `@mtchat/vue` и `@mtchat/vue-primevue` в консистентное состояние: README, docs, package metadata, примеры интеграции и экспортируемые контракты должны описывать один и тот же продукт без артефактов старой модели.

## Problem

Сейчас проект выглядит зрелым внутри, но наружу транслирует противоречивую картину:

- `mtchat-vue/README.md` описывает устаревшую модель с `employeeId`, `Tenant`, `Employee` и API, которого больше нет
- docs в `docs/sdk/*` уже описывают актуальную модель `userId + scopeConfig`
- `@mtchat/vue-primevue` содержит некорректный `repository.url`
- есть несколько источников правды для установки, quick start и SDK API

Это создаёт лишнюю стоимость интеграции и подрывает доверие к npm-пакетам.

## Goals

- README пакетов соответствует реальному API
- docs и README не дублируют противоречивые примеры
- package metadata готова к публикации и не содержит заглушек
- примеры использования проверяются минимальным smoke-контуром

## Non-Goals

- Полный редизайн документации
- Добавление новых SDK-фич
- Переписывание docs engine

## Scope

### 1. `@mtchat/vue`

- Переписать `mtchat-vue/README.md` под текущую object-bound архитектуру
- Удалить все legacy references на tenants/employees
- Синхронизировать примеры `MTChat`, `MTChatClient`, `useChat`
- Явно описать обязательные поля `config`
- Добавить короткий раздел про auth responsibility host app

### 2. `@mtchat/vue-primevue`

- Исправить package metadata
- Добавить README с чётким позиционированием wrapper-пакета
- Описать peer dependencies и минимальный setup
- Проверить экспорт темы и способ подключения registry

### 3. Source of Truth

- Зафиксировать, какие страницы считаются canonical:
  - npm README для быстрого старта
  - `docs/sdk/*` для полной документации
- Минимизировать копипасту между README и docs
- Оставить в README только high-signal сценарии

## Deliverables

- Обновлённый `mtchat-vue/README.md`
- Новый или обновлённый `mtchat-vue-primevue/README.md`
- Исправленный `mtchat-vue-primevue/package.json`
- Актуализированные ссылки между README и `docs/sdk/*`
- Короткий checklist публикационной готовности пакетов

## Implementation

### Phase 1. Inventory and Contract Freeze

- Сопоставить реальный экспорт SDK с README и docs
- Зафиксировать текущую модель конфигурации и обязательные поля
- Составить список legacy терминов, которые нужно убрать

### Phase 2. Rewrite Public Entry Docs

- Переписать README для `@mtchat/vue`
- Обновить install, quick start, direct SDK usage и composable usage
- Добавить совместимые snippets под TypeScript

### Phase 3. PrimeVue Wrapper Alignment

- Исправить `repository.url` и package metadata
- Документировать `provideRegistry(primevueRegistry)` как основной сценарий
- Проверить, что wrapper docs не обещают несуществующее поведение

### Phase 4. Validation

- Проверить, что все snippets компилируемо правдоподобны
- Проверить ссылки между локальными docs
- Пройтись по npm-facing файлам как интегратор с нуля

## Task Checklist

- [x] Актуализировать `mtchat-vue/README.md`
- [x] Убрать legacy API references из README
- [x] Добавить краткое описание current auth model
- [x] Добавить/обновить README для `mtchat-vue-primevue`
- [x] Исправить `repository.url` в `mtchat-vue-primevue/package.json`
- [x] Проверить `exports`, install examples и theme import examples
- [x] Согласовать README с `docs/sdk/installation.md`, `configuration.md`, `primevue.md`
- [x] Сформировать публикационный checklist для SDK packages

## Validation

- `npm pack --dry-run` для обоих пакетов
- `npm run build` для `mtchat-vue`
- `npm run build` для `mtchat-vue-primevue`
- Ручная верификация примеров на соответствие текущим типам

## Risks

- README можно случайно сделать слишком подробным и снова создать второй источник правды
- При переписывании примеров легко зафиксировать не все обязательные поля config
- Если PrimeVue wrapper менялся эволюционно, часть сценариев может оказаться undocumented-but-used

## Done Criteria

- npm-facing README больше не содержит legacy сущностей
- package metadata корректна и пригодна для публикации
- docs и README не противоречат друг другу по базовым сценариям интеграции
- новый пользователь может подключить SDK без чтения исходников
