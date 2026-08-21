# BootForge — Brand & Design Guide

## Brand Identity

### Name
**BootForge** — a forge creates tools and shapes raw materials. BootForge takes raw ISO files and forges them into a polished, universal boot drive.

### Tagline
> **One Drive. Every OS.**

### Alternative Taglines
- Create. Boot. Repair.
- Your Universal Boot Drive.
- Drag. Drop. Boot.

---

## Logo

### Concept
The BootForge logo combines:
- **Anvil shape** — representing the forge (creation/crafting)
- **USB/drive silhouette** — representing the product
- **Boot arrow** — representing the boot process (power/startup)

### Logo Variants

| Variant | Usage |
|---------|-------|
| Full logo + wordmark | Marketing, website, splash screen |
| Icon only | App icon, favicon, small spaces |
| Horizontal | Navigation bar, header |
| Monochrome (white) | Dark backgrounds |
| Monochrome (dark) | Light backgrounds |

### Clear Space
- Minimum clear space around logo: 1x the height of the icon
- Never stretch, rotate, or distort the logo

---

## Color System

### Primary Palette

Based on a dark, developer-tool aesthetic with a vibrant accent.

```css
:root {
    /* Core */
    --color-background:     #0F172A;    /* Deep navy — main background */
    --color-surface:        #1E293B;    /* Slate — cards, panels */
    --color-surface-hover:  #334155;    /* Lighter slate — hover states */
    --color-border:         #475569;    /* Border color */
    
    /* Text */
    --color-foreground:     #F8FAFC;    /* Primary text — near white */
    --color-text-secondary: #94A3B8;    /* Secondary text — muted */
    --color-text-muted:     #64748B;    /* Tertiary text — very muted */
    
    /* Accent */
    --color-accent:         #22C55E;    /* Green — primary actions, success */
    --color-accent-hover:   #16A34A;    /* Darker green — hover */
    --color-accent-glow:    rgba(34, 197, 94, 0.15); /* Subtle glow */
    
    /* Semantic */
    --color-info:           #3B82F6;    /* Blue — informational */
    --color-warning:        #F59E0B;    /* Amber — warnings */
    --color-error:          #EF4444;    /* Red — errors, destructive */
    --color-success:        #22C55E;    /* Green — success */
    
    /* Special */
    --color-windows:        #0078D4;    /* Windows blue */
    --color-linux:          #FCC624;    /* Linux/Tux yellow */
    --color-macos:          #A2AAAD;    /* macOS silver */
    --color-rescue:         #F97316;    /* Rescue/utility orange */
}
```

### Color Usage

| Element | Color | Variable |
|---------|-------|----------|
| App background | Deep navy | `--color-background` |
| Cards, panels | Slate | `--color-surface` |
| Hover states | Lighter slate | `--color-surface-hover` |
| Primary text | Near white | `--color-foreground` |
| Secondary text | Muted gray | `--color-text-secondary` |
| Primary buttons | Green | `--color-accent` |
| Borders | Mid gray | `--color-border` |
| Error states | Red | `--color-error` |
| Progress bars | Green | `--color-accent` |
| Windows ISO badge | Windows blue | `--color-windows` |
| Linux ISO badge | Tux yellow | `--color-linux` |
| Rescue ISO badge | Orange | `--color-rescue` |

---

## Typography

### Font Family

**Inter** — Clean, modern, excellent readability at all sizes.

```css
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap');

:root {
    --font-family:    'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
    --font-mono:      'JetBrains Mono', 'SF Mono', 'Fira Code', 'Cascadia Code', monospace;
}
```

### Type Scale

| Token | Size | Weight | Line Height | Usage |
|-------|------|--------|-------------|-------|
| `--text-xs` | 11px | 400 | 1.4 | Labels, badges |
| `--text-sm` | 13px | 400 | 1.5 | Secondary text, captions |
| `--text-base` | 15px | 400 | 1.6 | Body text |
| `--text-md` | 16px | 500 | 1.5 | Emphasized body |
| `--text-lg` | 18px | 600 | 1.4 | Section headers |
| `--text-xl` | 22px | 600 | 1.3 | Page headers |
| `--text-2xl` | 28px | 700 | 1.2 | Main title |
| `--text-3xl` | 36px | 700 | 1.1 | Hero text |

```css
:root {
    --text-xs:   0.6875rem;  /* 11px */
    --text-sm:   0.8125rem;  /* 13px */
    --text-base: 0.9375rem;  /* 15px */
    --text-md:   1rem;       /* 16px */
    --text-lg:   1.125rem;   /* 18px */
    --text-xl:   1.375rem;   /* 22px */
    --text-2xl:  1.75rem;    /* 28px */
    --text-3xl:  2.25rem;    /* 36px */
}
```

### Monospace (for technical data)

Use **JetBrains Mono** or system monospace for:
- Hash values
- File paths
- Device paths
- Terminal output
- Technical details

---

## Spacing

### Spacing Scale (4px base)

```css
:root {
    --space-1:  4px;
    --space-2:  8px;
    --space-3:  12px;
    --space-4:  16px;
    --space-5:  20px;
    --space-6:  24px;
    --space-8:  32px;
    --space-10: 40px;
    --space-12: 48px;
    --space-16: 64px;
}
```

---

## Border Radius

```css
:root {
    --radius-sm:   4px;   /* Buttons, small elements */
    --radius-md:   8px;   /* Cards, panels */
    --radius-lg:   12px;  /* Large cards, modals */
    --radius-xl:   16px;  /* Full-width panels */
    --radius-full: 9999px; /* Pills, avatars */
}
```

---

## Shadows & Effects

```css
:root {
    /* Subtle elevation */
    --shadow-sm:    0 1px 2px rgba(0, 0, 0, 0.3);
    --shadow-md:    0 4px 6px rgba(0, 0, 0, 0.3);
    --shadow-lg:    0 10px 15px rgba(0, 0, 0, 0.3);
    --shadow-xl:    0 20px 25px rgba(0, 0, 0, 0.3);
    
    /* Accent glow */
    --glow-accent:  0 0 20px rgba(34, 197, 94, 0.15);
    --glow-error:   0 0 20px rgba(239, 68, 68, 0.15);
    --glow-info:    0 0 20px rgba(59, 130, 246, 0.15);
    
    /* Transitions */
    --transition-fast:   150ms ease;
    --transition-base:   250ms ease;
    --transition-slow:   400ms ease;
}
```

---

## Component Design Patterns

### ISO Card

```
┌──────────────────────────────────────────┐
│  🐧  Ubuntu 24.04 LTS          ✓ Ready  │
│                                          │
│  x86_64 • UEFI + BIOS • Secure Boot    │
│  5.7 GB • SHA256 verified               │
│                                          │
│  Compatibility: ████████████ EXCELLENT   │
│                                          │
│  [Test in VM]  [Remove]                  │
└──────────────────────────────────────────┘
```

**States**:
- **Analyzing** — Pulsing border, spinner, "Analyzing..."
- **Ready** — Green checkmark, full details
- **Warning** — Amber badge, show issue
- **Error** — Red badge, show error + action

### Device Panel

```
┌──────────────────────────────────────────┐
│  USB SSD                        ● Ready  │
│  SanDisk Extreme 256 GB                  │
│  USB 3.2 • 238 GB available             │
│                                          │
│  Health: ████████████████████ Excellent   │
└──────────────────────────────────────────┘
```

### Progress Bar

```
Creating Boot Drive...

Phase: Copying ISOs (3 of 5)
Current: ubuntu-24.04-desktop-amd64.iso

████████████████████░░░░░░░░░░ 68%

Speed: 450 MB/s • ETA: 2:15
```

### Primary Button

```css
.btn-primary {
    background: var(--color-accent);
    color: #000;
    font-weight: 600;
    padding: var(--space-3) var(--space-6);
    border-radius: var(--radius-md);
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.btn-primary:hover {
    background: var(--color-accent-hover);
    box-shadow: var(--glow-accent);
    transform: translateY(-1px);
}

.btn-primary:active {
    transform: translateY(0);
}

.btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
    box-shadow: none;
}
```

---

## Icons

### Icon System

Use **Lucide Icons** (SVG, consistent style):
- Stroke width: 1.5px
- Size: 20px (default), 16px (small), 24px (large)
- Color: inherit from text color

### Key Icons

| Action | Icon | Lucide Name |
|--------|------|-------------|
| Add ISO | + circle | `plus-circle` |
| Download | Arrow down | `download` |
| USB drive | Hard drive | `hard-drive` |
| Settings | Gear | `settings` |
| Check/Success | Check circle | `check-circle` |
| Error | X circle | `x-circle` |
| Warning | Alert triangle | `alert-triangle` |
| Info | Info circle | `info` |
| Delete/Remove | Trash | `trash-2` |
| Eject | Eject | `eject` |
| Play/Test | Play | `play` |
| Refresh | Refresh | `refresh-cw` |
| Search | Search | `search` |
| Folder | Folder | `folder` |
| File | File | `file` |
| Windows | (custom SVG) | — |
| Linux | (custom SVG) | — |
| macOS | (custom SVG) | — |

> **No emoji as icons.** Always use SVG icons from Lucide or custom SVGs.

---

## Animation

### Transitions

| Element | Duration | Easing | Property |
|---------|----------|--------|----------|
| Button hover | 150ms | ease | background, box-shadow, transform |
| Card hover | 200ms | ease | border-color, box-shadow |
| Panel expand | 300ms | ease-out | height, opacity |
| Progress bar | 200ms | linear | width |
| Toast appear | 250ms | ease-out | transform, opacity |
| Toast dismiss | 200ms | ease-in | transform, opacity |
| Modal open | 250ms | ease-out | transform, opacity |
| Modal close | 200ms | ease-in | transform, opacity |

### Loading States

- **ISO Analysis**: Pulsing ring animation around ISO card
- **Hash Verification**: Progress bar with percentage
- **Drive Write**: Multi-phase progress bar with current file info
- **Device Detection**: Subtle scan animation on device panel

### Micro-interactions

- Button press: `scale(0.98)` for 100ms
- Success: Green checkmark with fade-in + scale
- Error: Red shake animation (subtle horizontal wiggle)
- New device: Slide-in from right with fade

---

## Responsive (Window Sizes)

### Minimum Window Size
- Width: 600px
- Height: 500px

### Optimal Window Size
- Width: 900px
- Height: 700px

### Layout Adaptation

```
< 700px (compact):
┌──────────────┐
│  Device       │
│  Panel        │
├──────────────┤
│  ISO List     │
│  (scrollable) │
├──────────────┤
│  Actions      │
└──────────────┘

≥ 700px (standard):
┌──────────┬───────────────┐
│  Device   │  ISO List     │
│  Panel    │  (scrollable) │
│           │               │
│  Actions  │               │
│           │               │
└──────────┴───────────────┘
```

---

## Accessibility

### Requirements

| Category | Standard |
|----------|----------|
| Color contrast | 4.5:1 minimum (WCAG AA) |
| Focus indicators | 2px solid visible ring |
| Keyboard navigation | Full tab support |
| Screen reader | ARIA labels on all interactive elements |
| Reduced motion | Respect `prefers-reduced-motion` |
| Font scaling | Support system font size preferences |

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + O` | Add ISO file |
| `Cmd/Ctrl + D` | Download OS |
| `Cmd/Ctrl + Enter` | Create boot drive |
| `Cmd/Ctrl + ,` | Open settings |
| `Cmd/Ctrl + E` | Eject selected device |
| `Delete` | Remove selected ISO |
| `Space` | Toggle selected item |
| `Tab` | Navigate between elements |
