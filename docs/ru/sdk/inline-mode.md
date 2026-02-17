# Встроенный режим

Встроенный режим отображает один чат, привязанный к бизнес-объекту. Используйте для встраивания чата на страницы детализации объектов -- заказов, тендеров, рейсов.

## Использование

```vue
<template>
  <div class="order-detail">
    <h1>Заказ #{{ order.number }}</h1>

    <div style="height: 400px;">
      <MTChat
        :config="config"
        mode="inline"
        :object-id="order.id"
        object-type="order"
        theme="light"
      />
    </div>
  </div>
</template>
```

## Как это работает

1. SDK вызывает `GET /api/v1/dialogs/by-object/{objectType}/{objectId}` для поиска последнего диалога для объекта.
2. Если диалог существует и пользователь -- участник, сообщения загружаются сразу.
3. Если пользователь -- потенциальный участник (совпадение scope), показывается приглашение присоединиться.
4. Если диалога нет, отображается placeholder.

!!! note "Несколько диалогов на объект"
    Для одного объекта может существовать несколько диалогов. В inline-режиме SDK загружает **последний созданный**. Используйте полный режим для отображения всех диалогов объекта.

## Props

| Prop | Тип | По умолчанию | Описание |
|------|-----|--------------|----------|
| `config` | `MTChatConfig` | **обязателен** | Конфигурация SDK |
| `mode` | `'inline'` | -- | Должен быть `'inline'` |
| `objectId` | `string` | **обязателен** | ID бизнес-объекта |
| `objectType` | `string` | **обязателен** | Тип объекта (напр., `'order'`, `'tender'`) |
| `showHeader` | `boolean` | `true` | Показать заголовок |
| `theme` | `string` | `'light'` | Тема |

## Реактивная привязка

Измените `objectId` для переключения между объектами:

```vue
<script setup>
import { ref } from 'vue'
const selectedOrderId = ref('order-1')
</script>

<template>
  <select v-model="selectedOrderId">
    <option value="order-1">Заказ #1</option>
    <option value="order-2">Заказ #2</option>
  </select>

  <MTChat
    :config="config"
    mode="inline"
    :object-id="selectedOrderId"
    object-type="order"
  />
</template>
```

При изменении `objectId` SDK автоматически отключается от текущего диалога и загружает новый.

## Пример TMS-интерфейса

```vue
<template>
  <div class="tms-layout">
    <div class="tms-table">
      <table>
        <tr v-for="order in orders" :key="order.id"
            :class="{ active: order.id === selectedOrderId }"
            @click="selectedOrderId = order.id">
          <td>{{ order.number }}</td>
          <td>{{ order.status }}</td>
        </tr>
      </table>
    </div>

    <div class="tms-chat-panel">
      <MTChat v-if="selectedOrderId"
        :config="config" mode="inline"
        :object-id="selectedOrderId" object-type="order" />
      <p v-else>Выберите заказ для просмотра чата</p>
    </div>
  </div>
</template>
```
