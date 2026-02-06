<script setup lang="ts">
/**
 * MessageEditor - Rich text editor for chat messages
 *
 * Based on Tiptap with support for:
 * - Text formatting (bold, italic, underline, strikethrough)
 * - Links, lists, blockquotes, code
 * - @mentions
 * - Markdown shortcuts
 */

import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Underline from '@tiptap/extension-underline'
import Link from '@tiptap/extension-link'
import Placeholder from '@tiptap/extension-placeholder'
import Mention from '@tiptap/extension-mention'
import { useI18n } from '../../i18n'
import type { DialogParticipant } from '../../types'
import MentionList from './MentionList.vue'

const props = defineProps<{
  placeholder?: string
  disabled?: boolean
  participants?: DialogParticipant[]
  currentUserId?: string
}>()

const emit = defineEmits<{
  submit: [content: string]
  'update:isEmpty': [isEmpty: boolean]
}>()

const { t } = useI18n()

// Mention suggestion state
const mentionListRef = ref<InstanceType<typeof MentionList> | null>(null)
const showMentionList = ref(false)
const mentionQuery = ref('')
const mentionPosition = ref({ top: 0, left: 0 })

// Filter participants for mention suggestions
const filteredParticipants = computed(() => {
  if (!props.participants) return []
  const query = mentionQuery.value.toLowerCase()
  return props.participants
    .filter(p => p.user_id !== props.currentUserId) // Exclude self
    .filter(p => {
      const name = p.display_name?.toLowerCase() || ''
      const company = p.company?.toLowerCase() || ''
      return name.includes(query) || company.includes(query)
    })
    .slice(0, 5) // Limit to 5 suggestions
})

// Create Tiptap editor
const editor = useEditor({
  extensions: [
    StarterKit.configure({
      // Configure starter kit extensions
      heading: false, // No headings in chat
      horizontalRule: false,
      // Enable markdown shortcuts
      bold: { HTMLAttributes: { class: 'mtchat-bold' } },
      italic: { HTMLAttributes: { class: 'mtchat-italic' } },
      strike: { HTMLAttributes: { class: 'mtchat-strike' } },
      code: { HTMLAttributes: { class: 'mtchat-code' } },
      codeBlock: { HTMLAttributes: { class: 'mtchat-code-block' } },
      blockquote: { HTMLAttributes: { class: 'mtchat-blockquote' } },
      bulletList: { HTMLAttributes: { class: 'mtchat-bullet-list' } },
      orderedList: { HTMLAttributes: { class: 'mtchat-ordered-list' } },
    }),
    Underline,
    Link.configure({
      openOnClick: false,
      HTMLAttributes: {
        class: 'mtchat-link',
        rel: 'noopener noreferrer',
        target: '_blank',
      },
    }),
    Placeholder.configure({
      placeholder: () => props.placeholder || t.value.input.placeholder,
    }),
    Mention.configure({
      HTMLAttributes: {
        class: 'mtchat-mention',
      },
      suggestion: {
        char: '@',
        items: ({ query }) => {
          mentionQuery.value = query
          return filteredParticipants.value
        },
        render: () => {
          return {
            onStart: (props) => {
              showMentionList.value = true
              mentionPosition.value = props.clientRect?.() || { top: 0, left: 0 }
            },
            onUpdate: (props) => {
              mentionPosition.value = props.clientRect?.() || { top: 0, left: 0 }
            },
            onKeyDown: (props) => {
              if (props.event.key === 'Escape') {
                showMentionList.value = false
                return true
              }
              if (props.event.key === 'ArrowUp') {
                mentionListRef.value?.moveUp()
                return true
              }
              if (props.event.key === 'ArrowDown') {
                mentionListRef.value?.moveDown()
                return true
              }
              if (props.event.key === 'Enter') {
                mentionListRef.value?.select()
                return true
              }
              return false
            },
            onExit: () => {
              showMentionList.value = false
            },
          }
        },
        command: ({ editor, range, props: mentionProps }) => {
          // Props contains DialogParticipant data
          const participant = mentionProps as unknown as DialogParticipant
          editor
            .chain()
            .focus()
            .insertContentAt(range, [
              {
                type: 'mention',
                attrs: {
                  id: participant.user_id,
                  label: participant.display_name || 'User',
                },
              },
              { type: 'text', text: ' ' },
            ])
            .run()
        },
      },
    }),
  ],
  editorProps: {
    attributes: {
      class: 'mtchat-editor__content',
    },
    handleKeyDown: (_view, event) => {
      // Submit on Enter (without Shift)
      if (event.key === 'Enter' && !event.shiftKey && !showMentionList.value) {
        event.preventDefault()
        handleSubmit()
        return true
      }
      return false
    },
  },
  onUpdate: ({ editor }) => {
    emit('update:isEmpty', editor.isEmpty)
  },
})

// Toolbar state
const showToolbar = ref(true)

// Check if format is active
const isActive = (name: string, attrs?: Record<string, unknown>) => {
  return editor.value?.isActive(name, attrs) || false
}

// Format commands
const toggleBold = () => editor.value?.chain().focus().toggleBold().run()
const toggleItalic = () => editor.value?.chain().focus().toggleItalic().run()
const toggleUnderline = () => editor.value?.chain().focus().toggleUnderline().run()
const toggleStrike = () => editor.value?.chain().focus().toggleStrike().run()
const toggleCode = () => editor.value?.chain().focus().toggleCode().run()
const toggleCodeBlock = () => editor.value?.chain().focus().toggleCodeBlock().run()
const toggleBulletList = () => editor.value?.chain().focus().toggleBulletList().run()
const toggleOrderedList = () => editor.value?.chain().focus().toggleOrderedList().run()
const toggleBlockquote = () => editor.value?.chain().focus().toggleBlockquote().run()
const clearFormatting = () => editor.value?.chain().focus().clearNodes().unsetAllMarks().run()

// Link handling
const linkUrl = ref('')
const showLinkDialog = ref(false)

const openLinkDialog = () => {
  const previousUrl = editor.value?.getAttributes('link').href || ''
  linkUrl.value = previousUrl
  showLinkDialog.value = true
}

const setLink = () => {
  if (linkUrl.value) {
    // Add https:// if no protocol
    let url = linkUrl.value
    if (!/^https?:\/\//i.test(url)) {
      url = 'https://' + url
    }
    editor.value?.chain().focus().extendMarkRange('link').setLink({ href: url }).run()
  } else {
    editor.value?.chain().focus().extendMarkRange('link').unsetLink().run()
  }
  showLinkDialog.value = false
  linkUrl.value = ''
}

const cancelLink = () => {
  showLinkDialog.value = false
  linkUrl.value = ''
  editor.value?.chain().focus().run()
}

// Submit handler
const handleSubmit = () => {
  if (!editor.value || editor.value.isEmpty) return

  const html = editor.value.getHTML()
  emit('submit', html)
  editor.value.commands.clearContent()
}

// Select mention from list
const selectMention = (participant: DialogParticipant) => {
  if (!editor.value) return

  editor.value
    .chain()
    .focus()
    .insertContent([
      {
        type: 'mention',
        attrs: {
          id: participant.user_id,
          label: participant.display_name || 'User',
        },
      },
      { type: 'text', text: ' ' },
    ])
    .run()

  showMentionList.value = false
}

// Focus the editor
const focus = () => {
  editor.value?.chain().focus().run()
}

// Clear content
const clear = () => {
  editor.value?.commands.clearContent()
}

// Keyboard shortcuts
const handleKeydown = (e: KeyboardEvent) => {
  if (!editor.value?.isFocused) return

  const isMod = e.metaKey || e.ctrlKey

  if (isMod && e.key === 'b') {
    e.preventDefault()
    toggleBold()
  } else if (isMod && e.key === 'i') {
    e.preventDefault()
    toggleItalic()
  } else if (isMod && e.key === 'u') {
    e.preventDefault()
    toggleUnderline()
  } else if (isMod && e.shiftKey && e.key === 's') {
    e.preventDefault()
    toggleStrike()
  } else if (isMod && e.key === 'k') {
    e.preventDefault()
    openLinkDialog()
  } else if (isMod && e.key === 'e') {
    e.preventDefault()
    toggleCode()
  } else if (isMod && e.shiftKey && e.key === 'c') {
    e.preventDefault()
    toggleCodeBlock()
  } else if (isMod && e.key === '\\') {
    e.preventDefault()
    clearFormatting()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeydown)
  editor.value?.destroy()
})

// Watch for disabled state
watch(() => props.disabled, (disabled) => {
  editor.value?.setEditable(!disabled)
})

// Expose methods
defineExpose({
  focus,
  clear,
  editor,
})
</script>

<template>
  <div class="mtchat-editor" :class="{ 'mtchat-editor--disabled': disabled }">
    <!-- Toolbar -->
    <div v-if="showToolbar" class="mtchat-editor__toolbar">
      <div class="mtchat-editor__toolbar-group">
        <button
          type="button"
          class="mtchat-editor__btn"
          :class="{ 'mtchat-editor__btn--active': isActive('bold') }"
          :title="`${t.formatting.bold} (⌘B)`"
          @click="toggleBold"
        >
          <strong>B</strong>
        </button>
        <button
          type="button"
          class="mtchat-editor__btn"
          :class="{ 'mtchat-editor__btn--active': isActive('italic') }"
          :title="`${t.formatting.italic} (⌘I)`"
          @click="toggleItalic"
        >
          <em>I</em>
        </button>
        <button
          type="button"
          class="mtchat-editor__btn"
          :class="{ 'mtchat-editor__btn--active': isActive('underline') }"
          :title="`${t.formatting.underline} (⌘U)`"
          @click="toggleUnderline"
        >
          <u>U</u>
        </button>
        <button
          type="button"
          class="mtchat-editor__btn"
          :class="{ 'mtchat-editor__btn--active': isActive('strike') }"
          :title="`${t.formatting.strikethrough} (⌘⇧S)`"
          @click="toggleStrike"
        >
          <s>S</s>
        </button>
      </div>

      <div class="mtchat-editor__toolbar-separator"></div>

      <div class="mtchat-editor__toolbar-group">
        <button
          type="button"
          class="mtchat-editor__btn"
          :class="{ 'mtchat-editor__btn--active': isActive('link') }"
          :title="`${t.formatting.link} (⌘K)`"
          @click="openLinkDialog"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
            <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
          </svg>
        </button>
      </div>

      <div class="mtchat-editor__toolbar-separator"></div>

      <div class="mtchat-editor__toolbar-group">
        <button
          type="button"
          class="mtchat-editor__btn"
          :class="{ 'mtchat-editor__btn--active': isActive('bulletList') }"
          :title="t.formatting.bulletList"
          @click="toggleBulletList"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="8" y1="6" x2="21" y2="6"/>
            <line x1="8" y1="12" x2="21" y2="12"/>
            <line x1="8" y1="18" x2="21" y2="18"/>
            <circle cx="4" cy="6" r="1" fill="currentColor"/>
            <circle cx="4" cy="12" r="1" fill="currentColor"/>
            <circle cx="4" cy="18" r="1" fill="currentColor"/>
          </svg>
        </button>
        <button
          type="button"
          class="mtchat-editor__btn"
          :class="{ 'mtchat-editor__btn--active': isActive('orderedList') }"
          :title="t.formatting.numberedList"
          @click="toggleOrderedList"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="10" y1="6" x2="21" y2="6"/>
            <line x1="10" y1="12" x2="21" y2="12"/>
            <line x1="10" y1="18" x2="21" y2="18"/>
            <text x="3" y="8" font-size="8" fill="currentColor">1</text>
            <text x="3" y="14" font-size="8" fill="currentColor">2</text>
            <text x="3" y="20" font-size="8" fill="currentColor">3</text>
          </svg>
        </button>
      </div>

      <div class="mtchat-editor__toolbar-separator"></div>

      <div class="mtchat-editor__toolbar-group">
        <button
          type="button"
          class="mtchat-editor__btn"
          :class="{ 'mtchat-editor__btn--active': isActive('blockquote') }"
          :title="t.formatting.quote"
          @click="toggleBlockquote"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 17h3l2-4V7H5v6h3zm8 0h3l2-4V7h-6v6h3z"/>
          </svg>
        </button>
      </div>

      <div class="mtchat-editor__toolbar-separator"></div>

      <div class="mtchat-editor__toolbar-group">
        <button
          type="button"
          class="mtchat-editor__btn"
          :class="{ 'mtchat-editor__btn--active': isActive('code') }"
          :title="`${t.formatting.inlineCode} (⌘E)`"
          @click="toggleCode"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="16 18 22 12 16 6"/>
            <polyline points="8 6 2 12 8 18"/>
          </svg>
        </button>
        <button
          type="button"
          class="mtchat-editor__btn"
          :class="{ 'mtchat-editor__btn--active': isActive('codeBlock') }"
          :title="`${t.formatting.codeBlock} (⌘⇧C)`"
          @click="toggleCodeBlock"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <polyline points="9 9 6 12 9 15"/>
            <polyline points="15 9 18 12 15 15"/>
          </svg>
        </button>
      </div>

      <div class="mtchat-editor__toolbar-separator"></div>

      <div class="mtchat-editor__toolbar-group">
        <button
          type="button"
          class="mtchat-editor__btn"
          :title="`${t.formatting.clearFormatting} (⌘\\)`"
          @click="clearFormatting"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M17 12H3"/>
            <path d="M21 6H3"/>
            <path d="M21 18H3"/>
            <path d="M19 4L5 20" stroke-width="2.5"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Editor Content -->
    <div class="mtchat-editor__wrapper">
      <EditorContent :editor="editor" class="mtchat-editor__input" />
    </div>

    <!-- Link Dialog -->
    <div v-if="showLinkDialog" class="mtchat-editor__link-dialog">
      <div class="mtchat-editor__link-dialog-backdrop" @click="cancelLink"></div>
      <div class="mtchat-editor__link-dialog-content">
        <input
          v-model="linkUrl"
          type="url"
          placeholder="https://example.com"
          class="mtchat-editor__link-input"
          @keydown.enter="setLink"
          @keydown.esc="cancelLink"
          autofocus
        />
        <div class="mtchat-editor__link-actions">
          <button type="button" class="mtchat-editor__link-btn" @click="cancelLink">
            {{ t.buttons.cancel }}
          </button>
          <button type="button" class="mtchat-editor__link-btn mtchat-editor__link-btn--primary" @click="setLink">
            OK
          </button>
        </div>
      </div>
    </div>

    <!-- Mention List -->
    <MentionList
      v-if="showMentionList && filteredParticipants.length > 0"
      ref="mentionListRef"
      :items="filteredParticipants"
      :style="{
        position: 'fixed',
        top: `${mentionPosition.top - 10}px`,
        left: `${mentionPosition.left}px`,
        transform: 'translateY(-100%)',
      }"
      @select="selectMention"
    />
  </div>
</template>

<style scoped>
.mtchat-editor {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--mtchat-border);
  border-radius: var(--mtchat-border-radius-md, 8px);
  background: var(--mtchat-bg);
  overflow: hidden;
}

.mtchat-editor--disabled {
  opacity: 0.6;
  pointer-events: none;
}

/* Toolbar */
.mtchat-editor__toolbar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 8px;
  background: var(--mtchat-bg-secondary);
  border-bottom: 1px solid var(--mtchat-border);
  flex-wrap: wrap;
}

.mtchat-editor__toolbar-group {
  display: flex;
  gap: 2px;
}

.mtchat-editor__toolbar-separator {
  width: 1px;
  height: 20px;
  background: var(--mtchat-border);
  margin: 0 4px;
}

.mtchat-editor__btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  color: var(--mtchat-text-secondary);
  font-size: 14px;
  transition: all 0.15s;
}

.mtchat-editor__btn:hover {
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-text);
}

.mtchat-editor__btn--active {
  background: var(--mtchat-primary);
  color: white;
}

.mtchat-editor__btn--active:hover {
  background: var(--mtchat-primary-hover);
  color: white;
}

/* Editor wrapper */
.mtchat-editor__wrapper {
  flex: 1;
  min-height: 44px;
  max-height: 25vh;
  overflow-y: auto;
}

.mtchat-editor__input {
  height: 100%;
}

/* Editor content styles */
.mtchat-editor__input :deep(.mtchat-editor__content) {
  padding: 12px 16px;
  min-height: 44px;
  outline: none;
  font-size: 14px;
  line-height: 1.5;
  color: var(--mtchat-text);
}

.mtchat-editor__input :deep(.mtchat-editor__content p) {
  margin: 0;
}

.mtchat-editor__input :deep(.mtchat-editor__content p + p) {
  margin-top: 8px;
}

.mtchat-editor__input :deep(.mtchat-editor__content p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  color: var(--mtchat-text-secondary);
  pointer-events: none;
  float: left;
  height: 0;
}

/* Formatting styles */
.mtchat-editor__input :deep(.mtchat-bold) {
  font-weight: 600;
}

.mtchat-editor__input :deep(.mtchat-italic) {
  font-style: italic;
}

.mtchat-editor__input :deep(.mtchat-strike) {
  text-decoration: line-through;
}

.mtchat-editor__input :deep(.mtchat-code) {
  font-family: 'SF Mono', Monaco, Menlo, monospace;
  font-size: 13px;
  background: var(--mtchat-bg-secondary);
  padding: 2px 6px;
  border-radius: 4px;
}

.mtchat-editor__input :deep(.mtchat-code-block) {
  font-family: 'SF Mono', Monaco, Menlo, monospace;
  font-size: 13px;
  background: var(--mtchat-bg-secondary);
  padding: 12px 16px;
  border-radius: 6px;
  margin: 8px 0;
  overflow-x: auto;
}

.mtchat-editor__input :deep(.mtchat-blockquote) {
  border-left: 3px solid var(--mtchat-primary);
  padding-left: 12px;
  margin: 8px 0;
  color: var(--mtchat-text-secondary);
}

.mtchat-editor__input :deep(.mtchat-bullet-list),
.mtchat-editor__input :deep(.mtchat-ordered-list) {
  padding-left: 24px;
  margin: 8px 0;
}

.mtchat-editor__input :deep(.mtchat-link) {
  color: var(--mtchat-primary);
  text-decoration: underline;
  cursor: pointer;
}

.mtchat-editor__input :deep(.mtchat-mention) {
  color: var(--mtchat-primary);
  background: rgba(59, 130, 246, 0.1);
  padding: 2px 4px;
  border-radius: 4px;
  font-weight: 500;
}

/* Link Dialog */
.mtchat-editor__link-dialog {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.mtchat-editor__link-dialog-backdrop {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
}

.mtchat-editor__link-dialog-content {
  position: relative;
  background: var(--mtchat-bg);
  border-radius: 8px;
  padding: 16px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
  min-width: 300px;
}

.mtchat-editor__link-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--mtchat-border);
  border-radius: 6px;
  font-size: 14px;
  background: var(--mtchat-bg);
  color: var(--mtchat-text);
  outline: none;
}

.mtchat-editor__link-input:focus {
  border-color: var(--mtchat-primary);
}

.mtchat-editor__link-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 12px;
}

.mtchat-editor__link-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  background: var(--mtchat-bg-secondary);
  color: var(--mtchat-text);
}

.mtchat-editor__link-btn:hover {
  background: var(--mtchat-bg-hover);
}

.mtchat-editor__link-btn--primary {
  background: var(--mtchat-primary);
  color: white;
}

.mtchat-editor__link-btn--primary:hover {
  background: var(--mtchat-primary-hover);
}
</style>
