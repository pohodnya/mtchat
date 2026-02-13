/**
 * MTChat i18n Translations
 *
 * Supported languages: Russian (ru), English (en), Chinese (zh)
 */

export type Locale = 'ru' | 'en' | 'zh'

export interface TranslationStrings {
  tabs: {
    myChats: string
    available: string
  }
  status: {
    connected: string
    disconnected: string
  }
  actions: {
    reply: string
    edit: string
    delete: string
  }
  buttons: {
    join: string
    send: string
    cancel: string
    leaveChat: string
    info: string
    archive: string
    unarchive: string
    pin: string
    unpin: string
    muteNotifications: string
    unmuteNotifications: string
  }
  chat: {
    participants: string
    canJoin: string
    noActiveChats: string
    noAvailableChats: string
    noMessages: string
    selectChat: string
    noChatForObject: string
    newMessages: string
    messageDeleted: string
    messageLoading: string
    joinToSend: string
    joinRequired: string
    archived: string
    edited: string
    editing: string
    deleteConfirm: string
    loadingOlder: string
    loadingNewer: string
  }
  system: {
    chatCreated: string
    participantJoined: string
    participantLeft: string
  }
  input: {
    placeholder: string
    attachFiles: string
  }
  search: {
    placeholder: string
    noResults: string
  }
  tooltips: {
    chatInfo: string
    menu: string
    reply: string
    scrollDown: string
    close: string
    back: string
    muted: string
    openObject: string
  }
  formatting: {
    bold: string
    italic: string
    underline: string
    strikethrough: string
    link: string
    bulletList: string
    numberedList: string
    quote: string
    inlineCode: string
    codeBlock: string
    clearFormatting: string
  }
  dates: {
    today: string
    yesterday: string
  }
  user: {
    you: string
    youBadge: string
    creator: string
    anonymous: string
    defaultName: string
  }
  infoPanel: {
    title: string
    participants: string
    openObject: string
    objectTypes: {
      tender: string
      order: string
      route: string
    }
  }
  joinDialog: {
    title: string
    displayName: string
    company: string
    showContacts: string
    joining: string
  }
  fileViewer: {
    loading: string
    loadingPdf: string
    failedToLoad: string
    page: string
    zoomIn: string
    zoomOut: string
    resetZoom: string
    download: string
    previous: string
    next: string
    close: string
    fileTypes: {
      word: string
      excel: string
      powerpoint: string
      zip: string
      rar: string
      sevenZip: string
      gzip: string
      text: string
      csv: string
      json: string
      xml: string
      video: string
      audio: string
      file: string
    }
  }
  readReceipts: {
    read: string
    andMore: string
    readBy: string
  }
}

export const translations: Record<Locale, TranslationStrings> = {
  // Russian (default)
  ru: {
    tabs: {
      myChats: 'Участвую',
      available: 'Доступные',
    },
    status: {
      connected: 'Подключено',
      disconnected: 'Отключено',
    },
    actions: {
      reply: 'Ответить',
      edit: 'Редактировать',
      delete: 'Удалить',
    },
    buttons: {
      join: 'Присоединиться',
      send: 'Отправить',
      cancel: 'Отмена',
      leaveChat: 'Покинуть чат',
      info: 'Информация',
      archive: 'Архивировать',
      unarchive: 'Разархивировать',
      pin: 'Закрепить',
      unpin: 'Открепить',
      muteNotifications: 'Отключить уведомления',
      unmuteNotifications: 'Включить уведомления',
    },
    chat: {
      participants: '{count} участников',
      canJoin: 'Можно присоединиться',
      noActiveChats: 'Нет активных чатов',
      noAvailableChats: 'Нет доступных чатов',
      noMessages: 'Нет сообщений',
      selectChat: 'Выберите чат для начала переписки',
      noChatForObject: 'Нет чата для этого объекта',
      newMessages: 'Новые сообщения',
      messageDeleted: 'Сообщение удалено',
      messageLoading: '...',
      joinToSend: 'Присоединитесь к чату, чтобы отправлять сообщения',
      joinRequired: 'Присоединитесь к чату, чтобы просматривать сообщения',
      archived: 'Архивные',
      edited: 'ред.',
      editing: 'Редактирование',
      deleteConfirm: 'Удалить сообщение?',
      loadingOlder: 'Загрузка...',
      loadingNewer: 'Загрузка...',
    },
    system: {
      chatCreated: 'Чат создан с участниками: {participants}',
      participantJoined: '{name} присоединился к чату',
      participantLeft: '{name} покинул чат',
    },
    input: {
      placeholder: 'Введите сообщение... (Enter для отправки)',
      attachFiles: 'Прикрепить файлы',
    },
    search: {
      placeholder: 'Поиск по чатам...',
      noResults: 'Ничего не найдено',
    },
    tooltips: {
      chatInfo: 'Информация о чате',
      menu: 'Меню',
      reply: 'Ответить',
      scrollDown: 'Вниз',
      close: 'Закрыть',
      back: 'Назад',
      muted: 'Уведомления отключены',
      openObject: 'Открыть объект',
    },
    formatting: {
      bold: 'Жирный',
      italic: 'Курсив',
      underline: 'Подчёркнутый',
      strikethrough: 'Зачёркнутый',
      link: 'Ссылка',
      bulletList: 'Маркированный список',
      numberedList: 'Нумерованный список',
      quote: 'Цитата',
      inlineCode: 'Код',
      codeBlock: 'Блок кода',
      clearFormatting: 'Очистить форматирование',
    },
    dates: {
      today: 'Сегодня',
      yesterday: 'Вчера',
    },
    user: {
      you: 'Вы',
      youBadge: '(Вы)',
      creator: 'Создатель',
      anonymous: 'Сотрудник компании',
      defaultName: 'Пользователь',
    },
    infoPanel: {
      title: 'Информация о чате',
      participants: 'Участники',
      openObject: 'Открыть объект',
      objectTypes: {
        tender: 'Тендер',
        order: 'Заказ',
        route: 'Рейс',
      },
    },
    joinDialog: {
      title: 'Присоединиться к чату',
      displayName: 'Отображаемое имя',
      company: 'Компания',
      showContacts: 'Показать контакты',
      joining: 'Присоединение...',
    },
    fileViewer: {
      loading: 'Загрузка...',
      loadingPdf: 'Загрузка PDF...',
      failedToLoad: 'Не удалось загрузить PDF',
      page: 'стр.',
      zoomIn: 'Увеличить (+)',
      zoomOut: 'Уменьшить (−)',
      resetZoom: 'Сбросить масштаб',
      download: 'Скачать',
      previous: 'Предыдущий (←)',
      next: 'Следующий (→)',
      close: 'Закрыть (Esc)',
      fileTypes: {
        word: 'Документ Word',
        excel: 'Таблица Excel',
        powerpoint: 'Презентация PowerPoint',
        zip: 'ZIP архив',
        rar: 'RAR архив',
        sevenZip: '7-Zip архив',
        gzip: 'GZIP архив',
        text: 'Текстовый файл',
        csv: 'CSV файл',
        json: 'JSON файл',
        xml: 'XML файл',
        video: 'Видео',
        audio: 'Аудио',
        file: 'Файл',
      },
    },
    readReceipts: {
      read: 'Прочитано',
      andMore: 'и ещё {count}',
      readBy: 'Прочитали',
    },
  },

  // English
  en: {
    tabs: {
      myChats: 'My Chats',
      available: 'Available',
    },
    status: {
      connected: 'Connected',
      disconnected: 'Disconnected',
    },
    actions: {
      reply: 'Reply',
      edit: 'Edit',
      delete: 'Delete',
    },
    buttons: {
      join: 'Join',
      send: 'Send',
      cancel: 'Cancel',
      leaveChat: 'Leave Chat',
      info: 'Info',
      archive: 'Archive',
      unarchive: 'Unarchive',
      pin: 'Pin',
      unpin: 'Unpin',
      muteNotifications: 'Mute notifications',
      unmuteNotifications: 'Unmute notifications',
    },
    chat: {
      participants: '{count} participants',
      canJoin: 'Can Join',
      noActiveChats: 'No active chats',
      noAvailableChats: 'No available chats',
      noMessages: 'No messages yet',
      selectChat: 'Select a chat to start messaging',
      noChatForObject: 'No chat available for this object',
      newMessages: 'New messages',
      messageDeleted: 'Message deleted',
      messageLoading: '...',
      joinToSend: 'Join this chat to send messages',
      joinRequired: 'Join this chat to view messages',
      archived: 'Archived',
      edited: 'edited',
      editing: 'Editing',
      deleteConfirm: 'Delete message?',
      loadingOlder: 'Loading...',
      loadingNewer: 'Loading...',
    },
    system: {
      chatCreated: 'Chat created with participants: {participants}',
      participantJoined: '{name} joined the chat',
      participantLeft: '{name} left the chat',
    },
    input: {
      placeholder: 'Type a message... (Enter to send)',
      attachFiles: 'Attach files',
    },
    search: {
      placeholder: 'Search chats...',
      noResults: 'No results found',
    },
    tooltips: {
      chatInfo: 'Chat info',
      menu: 'Menu',
      reply: 'Reply',
      scrollDown: 'Scroll down',
      close: 'Close',
      back: 'Back',
      muted: 'Notifications muted',
      openObject: 'Open object',
    },
    formatting: {
      bold: 'Bold',
      italic: 'Italic',
      underline: 'Underline',
      strikethrough: 'Strikethrough',
      link: 'Link',
      bulletList: 'Bullet List',
      numberedList: 'Numbered List',
      quote: 'Quote',
      inlineCode: 'Inline Code',
      codeBlock: 'Code Block',
      clearFormatting: 'Clear Formatting',
    },
    dates: {
      today: 'Today',
      yesterday: 'Yesterday',
    },
    user: {
      you: 'You',
      youBadge: '(You)',
      creator: 'Creator',
      anonymous: 'Company employee',
      defaultName: 'User',
    },
    infoPanel: {
      title: 'Chat Info',
      participants: 'Participants',
      openObject: 'Open object',
      objectTypes: {
        tender: 'Tender',
        order: 'Order',
        route: 'Route',
      },
    },
    joinDialog: {
      title: 'Join Chat',
      displayName: 'Display Name',
      company: 'Company',
      showContacts: 'Show Contacts',
      joining: 'Joining...',
    },
    fileViewer: {
      loading: 'Loading...',
      loadingPdf: 'Loading PDF...',
      failedToLoad: 'Failed to load PDF',
      page: 'page',
      zoomIn: 'Zoom in (+)',
      zoomOut: 'Zoom out (−)',
      resetZoom: 'Reset zoom',
      download: 'Download',
      previous: 'Previous (←)',
      next: 'Next (→)',
      close: 'Close (Esc)',
      fileTypes: {
        word: 'Word Document',
        excel: 'Excel Spreadsheet',
        powerpoint: 'PowerPoint Presentation',
        zip: 'ZIP Archive',
        rar: 'RAR Archive',
        sevenZip: '7-Zip Archive',
        gzip: 'GZIP Archive',
        text: 'Text File',
        csv: 'CSV File',
        json: 'JSON File',
        xml: 'XML File',
        video: 'Video',
        audio: 'Audio',
        file: 'File',
      },
    },
    readReceipts: {
      read: 'Read',
      andMore: 'and {count} more',
      readBy: 'Read by',
    },
  },

  // Chinese (Simplified)
  zh: {
    tabs: {
      myChats: '我的聊天',
      available: '可加入',
    },
    status: {
      connected: '已连接',
      disconnected: '已断开',
    },
    actions: {
      reply: '回复',
      edit: '编辑',
      delete: '删除',
    },
    buttons: {
      join: '加入',
      send: '发送',
      cancel: '取消',
      leaveChat: '退出聊天',
      info: '信息',
      archive: '归档',
      unarchive: '取消归档',
      pin: '置顶',
      unpin: '取消置顶',
      muteNotifications: '关闭通知',
      unmuteNotifications: '开启通知',
    },
    chat: {
      participants: '{count} 位参与者',
      canJoin: '可加入',
      noActiveChats: '没有活跃的聊天',
      noAvailableChats: '没有可用的聊天',
      noMessages: '暂无消息',
      selectChat: '选择一个聊天开始对话',
      noChatForObject: '此对象没有可用的聊天',
      newMessages: '新消息',
      messageDeleted: '消息已删除',
      messageLoading: '...',
      joinToSend: '加入此聊天以发送消息',
      joinRequired: '加入此聊天以查看消息',
      archived: '已归档',
      edited: '已编辑',
      editing: '编辑中',
      deleteConfirm: '删除消息？',
      loadingOlder: '加载中...',
      loadingNewer: '加载中...',
    },
    system: {
      chatCreated: '聊天已创建，参与者：{participants}',
      participantJoined: '{name} 加入了聊天',
      participantLeft: '{name} 离开了聊天',
    },
    input: {
      placeholder: '输入消息... (Enter 发送)',
      attachFiles: '添加附件',
    },
    search: {
      placeholder: '搜索聊天...',
      noResults: '未找到结果',
    },
    tooltips: {
      chatInfo: '聊天信息',
      menu: '菜单',
      reply: '回复',
      scrollDown: '向下滚动',
      close: '关闭',
      back: '返回',
      muted: '通知已关闭',
      openObject: '打开对象',
    },
    formatting: {
      bold: '粗体',
      italic: '斜体',
      underline: '下划线',
      strikethrough: '删除线',
      link: '链接',
      bulletList: '项目符号列表',
      numberedList: '编号列表',
      quote: '引用',
      inlineCode: '行内代码',
      codeBlock: '代码块',
      clearFormatting: '清除格式',
    },
    dates: {
      today: '今天',
      yesterday: '昨天',
    },
    user: {
      you: '您',
      youBadge: '(您)',
      creator: '创建者',
      anonymous: '公司员工',
      defaultName: '用户',
    },
    infoPanel: {
      title: '聊天信息',
      participants: '参与者',
      openObject: '打开对象',
      objectTypes: {
        tender: '招标',
        order: '订单',
        route: '路线',
      },
    },
    joinDialog: {
      title: '加入聊天',
      displayName: '显示名称',
      company: '公司',
      showContacts: '显示联系方式',
      joining: '加入中...',
    },
    fileViewer: {
      loading: '加载中...',
      loadingPdf: '加载PDF中...',
      failedToLoad: '加载PDF失败',
      page: '页',
      zoomIn: '放大 (+)',
      zoomOut: '缩小 (−)',
      resetZoom: '重置缩放',
      download: '下载',
      previous: '上一个 (←)',
      next: '下一个 (→)',
      close: '关闭 (Esc)',
      fileTypes: {
        word: 'Word文档',
        excel: 'Excel表格',
        powerpoint: 'PowerPoint演示文稿',
        zip: 'ZIP压缩包',
        rar: 'RAR压缩包',
        sevenZip: '7-Zip压缩包',
        gzip: 'GZIP压缩包',
        text: '文本文件',
        csv: 'CSV文件',
        json: 'JSON文件',
        xml: 'XML文件',
        video: '视频',
        audio: '音频',
        file: '文件',
      },
    },
    readReceipts: {
      read: '已读',
      andMore: '还有 {count} 人',
      readBy: '已读',
    },
  },
}
