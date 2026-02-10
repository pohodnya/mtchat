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

import { ref, watch, onBeforeUnmount, computed } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Underline from '@tiptap/extension-underline'
import Link from '@tiptap/extension-link'
import Placeholder from '@tiptap/extension-placeholder'
import Mention from '@tiptap/extension-mention'
import { Extension } from '@tiptap/core'
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
  attach: []
  'arrow-up': []
}>()

const { t } = useI18n()

// Mention suggestion state
const mentionListRef = ref<InstanceType<typeof MentionList> | null>(null)
const showMentionList = ref(false)
const mentionQuery = ref('')
const mentionPosition = ref({ top: 0, left: 0 })
const mentionCommandFn = ref<((props: { id: string; label: string }) => void) | null>(null)

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

// Submit handler reference for extension
const handleSubmitRef = ref<() => void>(() => {})

// Custom keyboard shortcuts extension
const CustomKeyboardShortcuts = Extension.create({
  name: 'customKeyboardShortcuts',

  addKeyboardShortcuts() {
    return {
      // Bold - Mod+B (StarterKit has this, but let's ensure it works)
      'Mod-b': () => this.editor.chain().focus().toggleBold().run(),
      // Italic - Mod+I
      'Mod-i': () => this.editor.chain().focus().toggleItalic().run(),
      // Underline - Mod+U (not in StarterKit)
      'Mod-u': () => this.editor.chain().focus().toggleUnderline().run(),
      // Strikethrough - Mod+Shift+S
      'Mod-Shift-s': () => this.editor.chain().focus().toggleStrike().run(),
      // Link - Mod+Shift+L (Cmd+Shift+L on Mac, Ctrl+Shift+L on Windows/Linux)
      'Mod-Shift-l': () => {
        openLinkDialog()
        return true
      },
      // Inline code - Mod+E
      'Mod-e': () => this.editor.chain().focus().toggleCode().run(),
      // Code block - Mod+Shift+C
      'Mod-Shift-c': () => this.editor.chain().focus().toggleCodeBlock().run(),
      // Clear formatting - Mod+\
      'Mod-\\': () => this.editor.chain().focus().clearNodes().unsetAllMarks().run(),
      // Submit on Cmd+Enter (Ctrl+Enter on Windows)
      'Mod-Enter': () => {
        // Don't handle if mention list is showing
        if (showMentionList.value) return false
        handleSubmitRef.value()
        return true
      },
      // Arrow Up when empty - edit last message
      'ArrowUp': () => {
        // Only trigger when editor is empty and cursor is at the start
        if (isEmpty.value) {
          emit('arrow-up')
          return true
        }
        return false
      },
    }
  },
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
    Mention.extend({
      // Add fallback parseHTML for legacy mentions (class only, no data-type)
      parseHTML() {
        return [
          { tag: `span[data-type="${this.name}"]` },
          // Fallback for legacy mentions without data-type
          {
            tag: 'span.mtchat-mention',
            getAttrs: (element) => {
              const el = element as HTMLElement
              // Only match if there's no data-type (legacy format)
              if (el.getAttribute('data-type')) return false
              return {
                id: el.getAttribute('data-id') || el.textContent?.replace(/^@/, '') || '',
                label: el.getAttribute('data-label') || el.textContent?.replace(/^@/, '') || '',
              }
            },
          },
        ]
      },
    }).configure({
      HTMLAttributes: {
        class: 'mtchat-mention',
      },
      renderLabel: ({ options, node }) => {
        return `${options.suggestion?.char ?? '@'}${node.attrs.label ?? node.attrs.id}`
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
              mentionCommandFn.value = props.command
            },
            onUpdate: (props) => {
              mentionPosition.value = props.clientRect?.() || { top: 0, left: 0 }
              mentionCommandFn.value = props.command
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
              mentionCommandFn.value = null
            },
          }
        },
        command: ({ editor, range, props: mentionProps }) => {
          // Props contains { id, label } from selectMention
          const { id, label } = mentionProps as { id: string; label: string }
          editor
            .chain()
            .focus()
            .insertContentAt(range, [
              {
                type: 'mention',
                attrs: { id, label },
              },
              { type: 'text', text: ' ' },
            ])
            .run()
        },
      },
    }),
    CustomKeyboardShortcuts,
  ],
  editorProps: {
    attributes: {
      class: 'mtchat-editor__content',
    },
  },
  onUpdate: ({ editor }) => {
    emit('update:isEmpty', editor.isEmpty)
  },
})

// Toolbar state
const showToolbar = ref(true)
const showMoreMenu = ref(false)

const toggleMoreMenu = () => {
  showMoreMenu.value = !showMoreMenu.value
}

const closeMoreMenu = () => {
  showMoreMenu.value = false
}

// Check if format is active
const isActive = (name: string, attrs?: Record<string, unknown>) => {
  return editor.value?.isActive(name, attrs) || false
}

// Check if editor has content
const isEmpty = computed(() => editor.value?.isEmpty ?? true)

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

// Update the ref for use in extension
handleSubmitRef.value = handleSubmit

// Attach button handler
const handleAttach = () => {
  emit('attach')
}

// Select mention from list
const selectMention = (participant: DialogParticipant) => {
  if (!mentionCommandFn.value) return

  // Use Tiptap's command function which properly handles the range
  mentionCommandFn.value({
    id: participant.user_id,
    label: participant.display_name || 'User',
  })

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

// Set content (for edit mode)
const setContent = (content: string) => {
  editor.value?.commands.setContent(content)
}

onBeforeUnmount(() => {
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
  setContent,
  editor,
  isEmpty,
})
</script>

<template>
  <div class="mtchat-editor" :class="{ 'mtchat-editor--disabled': disabled }">
    <!-- Toolbar -->
    <div v-if="showToolbar" class="mtchat-editor__toolbar">
      <!-- Primary: B I U S -->
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

      <!-- Link -->
      <button
        type="button"
        class="mtchat-editor__btn"
        :class="{ 'mtchat-editor__btn--active': isActive('link') }"
        :title="`${t.formatting.link} (⇧⌘L)`"
        @click="openLinkDialog"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/>
          <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>
        </svg>
      </button>

      <!-- Desktop: show all buttons inline -->
      <div class="mtchat-editor__toolbar-separator mtchat-editor__desktop-only"></div>

      <div class="mtchat-editor__toolbar-group mtchat-editor__desktop-only">
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

      <div class="mtchat-editor__toolbar-separator mtchat-editor__desktop-only"></div>

      <div class="mtchat-editor__toolbar-group mtchat-editor__desktop-only">
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

      <div class="mtchat-editor__toolbar-separator mtchat-editor__desktop-only"></div>

      <button
        type="button"
        class="mtchat-editor__btn mtchat-editor__desktop-only"
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

      <!-- Mobile: show more menu -->
      <div class="mtchat-editor__toolbar-separator mtchat-editor__mobile-only"></div>

      <div class="mtchat-editor__more-wrapper mtchat-editor__mobile-only">
        <button
          type="button"
          class="mtchat-editor__btn"
          :class="{ 'mtchat-editor__btn--active': showMoreMenu }"
          title="More formatting"
          @click="toggleMoreMenu"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
            <circle cx="5" cy="12" r="2"/>
            <circle cx="12" cy="12" r="2"/>
            <circle cx="19" cy="12" r="2"/>
          </svg>
        </button>

        <!-- Dropdown menu -->
        <div v-if="showMoreMenu" class="mtchat-editor__more-menu">
          <button
            type="button"
            :class="{ 'mtchat-editor__more-item--active': isActive('bulletList') }"
            @click="toggleBulletList(); closeMoreMenu()"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="8" y1="6" x2="21" y2="6"/>
              <line x1="8" y1="12" x2="21" y2="12"/>
              <line x1="8" y1="18" x2="21" y2="18"/>
              <circle cx="4" cy="6" r="1" fill="currentColor"/>
              <circle cx="4" cy="12" r="1" fill="currentColor"/>
              <circle cx="4" cy="18" r="1" fill="currentColor"/>
            </svg>
            {{ t.formatting.bulletList }}
          </button>
          <button
            type="button"
            :class="{ 'mtchat-editor__more-item--active': isActive('orderedList') }"
            @click="toggleOrderedList(); closeMoreMenu()"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="10" y1="6" x2="21" y2="6"/>
              <line x1="10" y1="12" x2="21" y2="12"/>
              <line x1="10" y1="18" x2="21" y2="18"/>
              <text x="3" y="8" font-size="8" fill="currentColor">1</text>
              <text x="3" y="14" font-size="8" fill="currentColor">2</text>
              <text x="3" y="20" font-size="8" fill="currentColor">3</text>
            </svg>
            {{ t.formatting.numberedList }}
          </button>
          <button
            type="button"
            :class="{ 'mtchat-editor__more-item--active': isActive('blockquote') }"
            @click="toggleBlockquote(); closeMoreMenu()"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
              <path d="M6 17h3l2-4V7H5v6h3zm8 0h3l2-4V7h-6v6h3z"/>
            </svg>
            {{ t.formatting.quote }}
          </button>
          <div class="mtchat-editor__more-separator"></div>
          <button
            type="button"
            :class="{ 'mtchat-editor__more-item--active': isActive('code') }"
            @click="toggleCode(); closeMoreMenu()"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="16 18 22 12 16 6"/>
              <polyline points="8 6 2 12 8 18"/>
            </svg>
            {{ t.formatting.inlineCode }}
          </button>
          <button
            type="button"
            :class="{ 'mtchat-editor__more-item--active': isActive('codeBlock') }"
            @click="toggleCodeBlock(); closeMoreMenu()"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="18" height="18" rx="2"/>
              <polyline points="9 9 6 12 9 15"/>
              <polyline points="15 9 18 12 15 15"/>
            </svg>
            {{ t.formatting.codeBlock }}
          </button>
          <div class="mtchat-editor__more-separator"></div>
          <button
            type="button"
            @click="clearFormatting(); closeMoreMenu()"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M17 12H3"/>
              <path d="M21 6H3"/>
              <path d="M21 18H3"/>
              <path d="M19 4L5 20" stroke-width="2.5"/>
            </svg>
            {{ t.formatting.clearFormatting }}
          </button>
        </div>

        <!-- Backdrop to close menu -->
        <div v-if="showMoreMenu" class="mtchat-editor__more-backdrop" @click="closeMoreMenu"></div>
      </div>
    </div>

    <!-- Editor Content -->
    <div class="mtchat-editor__wrapper">
      <EditorContent :editor="editor" class="mtchat-editor__input" />
    </div>

    <!-- Bottom Toolbar (attach + send) -->
    <div class="mtchat-editor__bottom">
      <button
        type="button"
        class="mtchat-editor__bottom-btn"
        :title="t.input.attachFiles"
        @click="handleAttach"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48" />
        </svg>
      </button>

      <div class="mtchat-editor__bottom-spacer"></div>

      <button
        type="button"
        class="mtchat-editor__send-btn"
        :class="{ 'mtchat-editor__send-btn--disabled': isEmpty }"
        :title="t.buttons.send"
        :disabled="isEmpty"
        @click="handleSubmit"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="22" y1="2" x2="11" y2="13"/>
          <polygon points="22 2 15 22 11 13 2 9 22 2"/>
        </svg>
      </button>
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
  gap: 2px;
  padding: 4px 8px;
  border-bottom: 1px solid var(--mtchat-border);
}

.mtchat-editor__toolbar-group {
  display: flex;
  gap: 1px;
}

.mtchat-editor__toolbar-separator {
  width: 1px;
  height: 18px;
  background: var(--mtchat-border);
  margin: 0 4px;
}

.mtchat-editor__btn {
  width: 28px;
  height: 28px;
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
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-text);
}

/* More menu wrapper */
.mtchat-editor__more-wrapper {
  position: relative;
}

.mtchat-editor__more-backdrop {
  position: fixed;
  inset: 0;
  z-index: 99;
}

.mtchat-editor__more-menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 4px;
  background: var(--mtchat-bg);
  border: 1px solid var(--mtchat-border);
  border-radius: 8px;
  box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.15);
  min-width: 180px;
  padding: 4px;
  z-index: 100;
}

.mtchat-editor__more-menu button {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  border-radius: 4px;
  font-size: 13px;
  color: var(--mtchat-text);
  cursor: pointer;
  text-align: left;
}

.mtchat-editor__more-menu button:hover {
  background: var(--mtchat-bg-hover);
}

.mtchat-editor__more-menu button svg {
  flex-shrink: 0;
  color: var(--mtchat-text-secondary);
}

.mtchat-editor__more-item--active {
  background: var(--mtchat-bg-hover);
}

.mtchat-editor__more-separator {
  height: 1px;
  background: var(--mtchat-border);
  margin: 4px 0;
}

/* Responsive toolbar using container queries */
.mtchat-editor {
  container-type: inline-size;
}

.mtchat-editor__desktop-only {
  display: flex;
}

.mtchat-editor__mobile-only {
  display: none;
}

@container (max-width: 400px) {
  .mtchat-editor__desktop-only {
    display: none !important;
  }

  .mtchat-editor__mobile-only {
    display: flex !important;
  }
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

/* Bottom toolbar */
.mtchat-editor__bottom {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  gap: 4px;
}

.mtchat-editor__bottom-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  color: var(--mtchat-text-secondary);
  transition: all 0.15s;
}

.mtchat-editor__bottom-btn:hover {
  background: var(--mtchat-bg-hover);
  color: var(--mtchat-text);
}

.mtchat-editor__bottom-spacer {
  flex: 1;
}

.mtchat-editor__send-btn {
  height: 28px;
  padding: 0 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: var(--mtchat-primary);
  color: white;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.mtchat-editor__send-btn:hover:not(:disabled) {
  background: var(--mtchat-primary-hover);
}

.mtchat-editor__send-btn--disabled,
.mtchat-editor__send-btn:disabled {
  background: transparent;
  color: var(--mtchat-text-secondary);
  cursor: not-allowed;
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
