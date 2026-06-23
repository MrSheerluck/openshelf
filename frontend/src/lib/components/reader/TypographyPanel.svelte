<script lang="ts">
  import type { FontFamily, ThemeName, Typography } from "$lib/reader/types";
  import { fontFamilies, themes } from "$lib/reader/themes";

  interface Props {
    typography: Typography;
    theme: ThemeName;
    progress: number;
    onChange: (typography: Typography) => void;
    onSetTheme: (theme: ThemeName) => void;
    onClose: () => void;
  }

  let { typography, theme, progress, onChange, onSetTheme, onClose }: Props =
    $props();

  function setFontFamily(family: FontFamily) {
    onChange({ ...typography, fontFamily: family });
  }

  function setFontSize(value: number) {
    onChange({ ...typography, fontSize: Math.max(80, Math.min(200, value)) });
  }

  function dotBorder(bg: string): string {
    if (bg === "#0a0a0a" || bg === "#1a1a1a") return "#444";
    if (bg === "#f4ecd8") return "#d4c9a8";
    if (bg === "#f7f3e9") return "#e0d9c4";
    if (bg === "#e4ede4") return "#c4d4c4";
    return "#ccc";
  }
</script>

<div class="typo-overlay" onclick={onClose} role="presentation">
  <div
    class="typo-panel"
    onclick={(e) => e.stopPropagation()}
    role="presentation"
  >
    <div class="typo-handle" aria-hidden="true"></div>

    <section class="typo-section">
      <div class="typo-label-row">
        <span class="typo-label">Font</span>
      </div>
      <div class="font-family">
        {#each fontFamilies as f}
          <button
            class="font-family-btn"
            class:active={typography.fontFamily === f.value}
            style:font-family={f.css}
            onclick={() => setFontFamily(f.value)}
          >
            {f.label}
          </button>
        {/each}
      </div>
    </section>

    <section class="typo-section">
      <div class="typo-label-row">
        <span class="typo-label">Size</span>
        <span class="typo-value">{typography.fontSize}%</span>
      </div>
      <div class="size-row">
        <button
          class="size-btn"
          onclick={() => setFontSize(typography.fontSize - 10)}
          aria-label="Decrease">A-</button
        >
        <div class="size-presets">
          {#each [80, 100, 120, 140, 160, 180, 200] as preset}
            <button
              class="preset-dot"
              class:active={typography.fontSize === preset}
              onclick={() => setFontSize(preset)}
              aria-label="{preset}%"
            ></button>
          {/each}
        </div>
        <button
          class="size-btn"
          onclick={() => setFontSize(typography.fontSize + 10)}
          aria-label="Increase">A+</button
        >
      </div>
    </section>

    <section class="typo-section">
      <div class="typo-label-row">
        <span class="typo-label">Theme</span>
      </div>
      <div class="theme-options">
        {#each themes as t}
          <button
            class="theme-option"
            class:active={theme === t.name}
            onclick={() => onSetTheme(t.name)}
            title={t.label}
          >
            <span
              class="theme-dot"
              style="background: {t.bg}; border-color: {dotBorder(t.bg)};"
            ></span>
            <span class="theme-label">{t.label}</span>
          </button>
        {/each}
      </div>
    </section>

    <section class="typo-section reading-progress">
      <div class="typo-label-row">
        <span class="typo-label">Progress</span>
        <span class="typo-value">{progress}%</span>
      </div>
      <div class="progress-bar-full">
        <div class="progress-bar-fill" style="width: {progress}%"></div>
      </div>
    </section>
  </div>
</div>

<style>
  .typo-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    z-index: 60;
    display: flex;
    align-items: flex-end;
    justify-content: center;
  }

  .typo-panel {
    width: 100%;
    max-width: 440px;
    background: var(--reader-panel-bg, #fff);
    color: var(--reader-panel-fg, #1a1a1a);
    border-top-left-radius: 16px;
    border-top-right-radius: 16px;
    padding: 0.5rem 1.25rem 2rem;
    box-shadow: 0 -8px 32px rgba(0, 0, 0, 0.2);
    animation: slide-up 0.22s ease-out;
    max-height: 85vh;
    overflow-y: auto;
  }

  @keyframes slide-up {
    from {
      transform: translateY(100%);
    }
    to {
      transform: translateY(0);
    }
  }

  .typo-handle {
    width: 36px;
    height: 4px;
    background: var(--reader-border, rgba(0, 0, 0, 0.2));
    border-radius: 2px;
    margin: 0 auto 1rem;
  }

  .typo-section {
    padding: 0.7rem 0;
    border-top: 1px solid var(--reader-border, rgba(0, 0, 0, 0.06));
  }
  .typo-section:first-of-type {
    border-top: none;
  }

  .typo-label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }

  .typo-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--reader-panel-fg, #1a1a1a);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .typo-value {
    font-size: 0.8rem;
    color: var(--reader-muted, #6b7280);
    font-variant-numeric: tabular-nums;
  }

  .font-family {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.4rem;
  }

  .font-family-btn {
    background: none;
    border: 1px solid var(--reader-border, rgba(0, 0, 0, 0.12));
    border-radius: 8px;
    padding: 0.6rem 0.5rem;
    cursor: pointer;
    color: var(--reader-panel-fg, #1a1a1a);
    font-size: 0.9rem;
    transition:
      border-color 0.15s,
      color 0.15s;
    text-align: center;
  }
  .font-family-btn:hover {
    background: var(--reader-hover, rgba(0, 0, 0, 0.04));
  }
  .font-family-btn.active {
    border-color: #4f46e5;
    color: #4f46e5;
    background: rgba(79, 70, 229, 0.06);
  }

  .size-row {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .size-btn {
    background: none;
    border: 1px solid var(--reader-border, rgba(0, 0, 0, 0.12));
    border-radius: 6px;
    padding: 0.35rem 0.55rem;
    cursor: pointer;
    color: var(--reader-panel-fg, #1a1a1a);
    font-size: 0.85rem;
    font-weight: 600;
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .size-btn:hover {
    background: var(--reader-hover, rgba(0, 0, 0, 0.04));
  }

  .size-presets {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex: 1;
    justify-content: center;
  }

  .preset-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    border: 1.5px solid var(--reader-border, rgba(0, 0, 0, 0.2));
    background: none;
    padding: 0;
    cursor: pointer;
    transition:
      background 0.15s,
      border-color 0.15s,
      transform 0.15s;
  }
  .preset-dot:nth-child(1) {
    width: 6px;
    height: 6px;
  }
  .preset-dot:nth-child(2) {
    width: 7px;
    height: 7px;
  }
  .preset-dot:nth-child(3) {
    width: 8px;
    height: 8px;
  }
  .preset-dot:nth-child(4) {
    width: 10px;
    height: 10px;
  }
  .preset-dot:nth-child(5) {
    width: 12px;
    height: 12px;
  }
  .preset-dot:nth-child(6) {
    width: 14px;
    height: 14px;
  }
  .preset-dot:nth-child(7) {
    width: 16px;
    height: 16px;
  }

  .preset-dot.active {
    background: #4f46e5;
    border-color: #4f46e5;
    transform: scale(1.1);
  }

  .theme-options {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
  }

  .theme-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.3rem;
    background: none;
    border: 1px solid var(--reader-border, rgba(0, 0, 0, 0.12));
    border-radius: 10px;
    padding: 0.6rem 0.5rem;
    cursor: pointer;
    transition: border-color 0.15s;
  }
  .theme-option:hover {
    background: var(--reader-hover, rgba(0, 0, 0, 0.04));
  }
  .theme-option.active {
    border-color: #4f46e5;
    background: rgba(79, 70, 229, 0.06);
  }

  .theme-dot {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid #ccc;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .theme-label {
    font-size: 0.7rem;
    color: var(--reader-panel-fg, #1a1a1a);
    opacity: 0.8;
  }

  .reading-progress {
    padding-bottom: 0;
  }

  .progress-bar-full {
    height: 4px;
    background: var(--reader-border, rgba(0, 0, 0, 0.08));
    border-radius: 2px;
    overflow: hidden;
    margin-top: 0.4rem;
  }

  .progress-bar-fill {
    height: 100%;
    background: #4f46e5;
    border-radius: 2px;
    transition: width 0.3s;
  }
</style>
