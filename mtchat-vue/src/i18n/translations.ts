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
  buttons: {
    join: string
    send: string
    cancel: string
    leaveChat: string
    info: string
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
    joinToSend: string
    joinRequired: string
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
}

export const translations: Record<Locale, TranslationStrings> = {
  // Russian (default)
  ru: {
    tabs: {
      myChats: 'Мои чаты',
      available: 'Доступные',
    },
    status: {
      connected: 'Подключено',
      disconnected: 'Отключено',
    },
    buttons: {
      join: 'Присоединиться',
      send: 'Отправить',
      cancel: 'Отмена',
      leaveChat: 'Покинуть чат',
      info: 'Информация',
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
      joinToSend: 'Присоединитесь к чату, чтобы отправлять сообщения',
      joinRequired: 'Присоединитесь к чату, чтобы просматривать сообщения',
    },
    input: {
      placeholder: 'Введите сообщение...',
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
    },
    dates: {
      today: 'Сегодня',
      yesterday: 'Вчера',
    },
    user: {
      you: 'Вы',
      youBadge: '(Вы)',
      creator: 'Создатель',
      anonymous: 'Сотрудник компании {company}',
      defaultName: 'Пользователь',
    },
    infoPanel: {
      title: 'Информация о чате',
      participants: 'Участники',
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
    buttons: {
      join: 'Join',
      send: 'Send',
      cancel: 'Cancel',
      leaveChat: 'Leave Chat',
      info: 'Info',
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
      joinToSend: 'Join this chat to send messages',
      joinRequired: 'Join this chat to view messages',
    },
    input: {
      placeholder: 'Type a message...',
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
    },
    dates: {
      today: 'Today',
      yesterday: 'Yesterday',
    },
    user: {
      you: 'You',
      youBadge: '(You)',
      creator: 'Creator',
      anonymous: 'Employee of {company}',
      defaultName: 'User',
    },
    infoPanel: {
      title: 'Chat Info',
      participants: 'Participants',
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
    buttons: {
      join: '加入',
      send: '发送',
      cancel: '取消',
      leaveChat: '退出聊天',
      info: '信息',
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
      joinToSend: '加入此聊天以发送消息',
      joinRequired: '加入此聊天以查看消息',
    },
    input: {
      placeholder: '输入消息...',
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
    },
    dates: {
      today: '今天',
      yesterday: '昨天',
    },
    user: {
      you: '您',
      youBadge: '(您)',
      creator: '创建者',
      anonymous: '{company}员工',
      defaultName: '用户',
    },
    infoPanel: {
      title: '聊天信息',
      participants: '参与者',
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
  },
}
