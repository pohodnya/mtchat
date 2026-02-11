# MTChat Icons

Все иконки в формате SVG, viewBox 24x24.

## Список иконок

| Файл | Описание | Использование |
|------|----------|---------------|
| `archive.svg` | Архив (коробка) | Архивирование чата |
| `attach.svg` | Скрепка | Прикрепить файл |
| `bell.svg` | Колокольчик | Уведомления включены |
| `bell-off.svg` | Колокольчик перечёркнутый | Уведомления выключены |
| `check.svg` | Галочка | Подтверждение, прочитано |
| `chevron-down.svg` | Стрелка вниз | Раскрыть |
| `chevron-left.svg` | Стрелка влево | Назад |
| `chevron-right.svg` | Стрелка вправо | Вперёд, развернуть |
| `close.svg` | Крестик | Закрыть |
| `code.svg` | Угловые скобки `<>` | Inline код |
| `code-block.svg` | Код в рамке | Блок кода |
| `download.svg` | Стрелка вниз в коробку | Скачать |
| `edit.svg` | Карандаш | Редактировать |
| `email.svg` | Конверт | Email |
| `error.svg` | Круг с восклицательным знаком | Ошибка |
| `external-link.svg` | Стрелка наружу | Внешняя ссылка |
| `info.svg` | Круг с i | Информация |
| `link.svg` | Цепь | Ссылка |
| `list-bullet.svg` | Маркированный список | Bullet list |
| `list-numbered.svg` | Нумерованный список | Numbered list |
| `lock.svg` | Замок | Заблокировано |
| `logout.svg` | Дверь со стрелкой | Выйти |
| `more-horizontal.svg` | Три точки горизонтально | Дополнительные действия |
| `more-vertical.svg` | Три точки вертикально | Меню сообщения |
| `phone.svg` | Телефон | Телефон |
| `pin.svg` | Булавка | Закреплённый чат |
| `quote.svg` | Кавычки | Цитата |
| `reply.svg` | Стрелка назад | Ответить |
| `reset.svg` | Круговая стрелка | Сбросить |
| `send.svg` | Бумажный самолётик | Отправить |
| `strikethrough.svg` | Зачёркнутый текст | Зачёркивание |
| `zoom-in.svg` | Лупа с плюсом | Увеличить |
| `zoom-out.svg` | Лупа с минусом | Уменьшить |

## Требования к иконкам

- Формат: SVG
- ViewBox: `0 0 24 24`
- Цвет: `currentColor` (наследуется от родителя)
- Stroke-based иконки: `stroke="currentColor" stroke-width="2"`
- Fill-based иконки: `fill="currentColor"`

## Использование

```vue
<script setup>
import Icon from '@/components/Icon.vue'
</script>

<template>
  <Icon name="close" :size="16" />
  <Icon name="bell" :size="20" />
</template>
```

## Замена иконок

Для замены иконок достаточно заменить SVG файлы в этой директории, сохраняя:
- Имена файлов
- ViewBox 24x24
- Использование currentColor для цвета
