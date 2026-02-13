<script setup lang="ts">
import { computed } from 'vue'

// Import all SVG icons
import closeIcon from '../icons/close.svg?raw'
import pinIcon from '../icons/pin.svg?raw'
import unpinIcon from '../icons/unpin.svg?raw'
import bellIcon from '../icons/bell.svg?raw'
import bellOffIcon from '../icons/bell-off.svg?raw'
import boldIcon from '../icons/bold.svg?raw'
import italicIcon from '../icons/italic.svg?raw'
import underlineIcon from '../icons/underline.svg?raw'
import chevronRightIcon from '../icons/chevron-right.svg?raw'
import chevronLeftIcon from '../icons/chevron-left.svg?raw'
import chevronDownIcon from '../icons/chevron-down.svg?raw'
import archiveIcon from '../icons/archive.svg?raw'
import externalLinkIcon from '../icons/external-link.svg?raw'
import moreVerticalIcon from '../icons/more-vertical.svg?raw'
import moreHorizontalIcon from '../icons/more-horizontal.svg?raw'
import infoIcon from '../icons/info.svg?raw'
import logoutIcon from '../icons/logout.svg?raw'
import lockIcon from '../icons/lock.svg?raw'
import replyIcon from '../icons/reply.svg?raw'
import editIcon from '../icons/edit.svg?raw'
import checkIcon from '../icons/check.svg?raw'
import zoomOutIcon from '../icons/zoom-out.svg?raw'
import zoomInIcon from '../icons/zoom-in.svg?raw'
import resetIcon from '../icons/reset.svg?raw'
import downloadIcon from '../icons/download.svg?raw'
import emailIcon from '../icons/email.svg?raw'
import phoneIcon from '../icons/phone.svg?raw'
import linkIcon from '../icons/link.svg?raw'
import listBulletIcon from '../icons/list-bullet.svg?raw'
import listNumberedIcon from '../icons/list-numbered.svg?raw'
import quoteIcon from '../icons/quote.svg?raw'
import codeIcon from '../icons/code.svg?raw'
import codeBlockIcon from '../icons/code-block.svg?raw'
import strikethroughIcon from '../icons/strikethrough.svg?raw'
import attachIcon from '../icons/attach.svg?raw'
import sendIcon from '../icons/send.svg?raw'
import errorIcon from '../icons/error.svg?raw'
import clearFormattingIcon from '../icons/clear-formatting.svg?raw'
import fileIcon from '../icons/file.svg?raw'
import fileDocumentIcon from '../icons/file-document.svg?raw'
import fileSpreadsheetIcon from '../icons/file-spreadsheet.svg?raw'
import fileArchiveIcon from '../icons/file-archive.svg?raw'
import fileVideoIcon from '../icons/file-video.svg?raw'
import fileAudioIcon from '../icons/file-audio.svg?raw'
import filePdfIcon from '../icons/file-pdf.svg?raw'
import imageIcon from '../icons/image.svg?raw'
import tableIcon from '../icons/table.svg?raw'
import printIcon from '../icons/print.svg?raw'

const icons: Record<string, string> = {
  close: closeIcon,
  pin: pinIcon,
  unpin: unpinIcon,
  bell: bellIcon,
  bold: boldIcon,
  italic: italicIcon,
  underline: underlineIcon,
  'bell-off': bellOffIcon,
  'chevron-right': chevronRightIcon,
  'chevron-left': chevronLeftIcon,
  'chevron-down': chevronDownIcon,
  archive: archiveIcon,
  'external-link': externalLinkIcon,
  'more-vertical': moreVerticalIcon,
  'more-horizontal': moreHorizontalIcon,
  info: infoIcon,
  logout: logoutIcon,
  lock: lockIcon,
  reply: replyIcon,
  edit: editIcon,
  check: checkIcon,
  'zoom-out': zoomOutIcon,
  'zoom-in': zoomInIcon,
  reset: resetIcon,
  download: downloadIcon,
  email: emailIcon,
  phone: phoneIcon,
  link: linkIcon,
  'list-bullet': listBulletIcon,
  'list-numbered': listNumberedIcon,
  quote: quoteIcon,
  code: codeIcon,
  'code-block': codeBlockIcon,
  strikethrough: strikethroughIcon,
  attach: attachIcon,
  send: sendIcon,
  error: errorIcon,
  'clear-formatting': clearFormattingIcon,
  file: fileIcon,
  'file-document': fileDocumentIcon,
  'file-spreadsheet': fileSpreadsheetIcon,
  'file-archive': fileArchiveIcon,
  'file-video': fileVideoIcon,
  'file-audio': fileAudioIcon,
  'file-pdf': filePdfIcon,
  image: imageIcon,
  table: tableIcon,
  print: printIcon,
}

export type IconName = keyof typeof icons

const props = withDefaults(defineProps<{
  name: IconName
  size?: number | string
}>(), {
  size: 24,
})

const svgContent = computed(() => {
  const raw = icons[props.name]
  if (!raw) {
    console.warn(`Icon "${props.name}" not found`)
    return ''
  }
  // Replace width/height in SVG with props.size
  return raw
    .replace(/width="\d+"/, `width="${props.size}"`)
    .replace(/height="\d+"/, `height="${props.size}"`)
})
</script>

<template>
  <span class="mtchat-icon" v-html="svgContent" />
</template>

<style scoped>
.mtchat-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.mtchat-icon :deep(svg) {
  display: block;
}
</style>
