<script lang="ts">
  interface DictEntry {
    word: string;
    phonetic?: string;
    meanings: {
      partOfSpeech: string;
      definitions: { definition: string; example?: string }[];
    }[];
  }

  interface Props {
    word: string;
    x: number;
    y: number;
    onClose: () => void;
  }

  let { word, x, y, onClose }: Props = $props();

  let loading = $state(true);
  let entries = $state<DictEntry[]>([]);
  let error = $state("");

  let style = $derived(() => {
    const w = 340;
    const left = Math.min(x, window.innerWidth - w - 16);
    const top = y + 12;
    return `left: ${Math.max(8, left)}px; top: ${Math.min(top, window.innerHeight - 280)}px; width: ${w}px;`;
  });

  async function lookup() {
    loading = true;
    error = "";
    entries = [];
    try {
      const res = await fetch(`https://api.dictionaryapi.dev/api/v2/entries/en/${encodeURIComponent(word)}`);
      if (res.ok) {
        const data = await res.json();
        entries = data.map((e: any) => ({
          word: e.word,
          phonetic: e.phonetic ?? e.phonetics?.find((p: any) => p.text)?.text,
          meanings: (e.meanings ?? []).map((m: any) => ({
            partOfSpeech: m.partOfSpeech,
            definitions: (m.definitions ?? []).slice(0, 3).map((d: any) => ({
              definition: d.definition,
              example: d.example,
            })),
          })),
        }));
        if (entries.length === 0) error = "No definition found.";
      } else {
        error = "No definition found.";
      }
    } catch {
      error = "Failed to look up word.";
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    lookup();
  });
</script>

<div class="popover-overlay" onclick={onClose} role="presentation">
<div
  class="dict-popup"
  style={style()}
  onclick={(e) => e.stopPropagation()}
  onkeydown={(e) => e.stopPropagation()}
  role="dialog"
  tabindex="-1"
>
  <div class="dict-header">
    <span class="dict-word">{word}</span>
    {#if entries[0]?.phonetic}
      <span class="dict-phonetic">{entries[0].phonetic}</span>
    {/if}
    <button class="dict-close" onclick={onClose} aria-label="Close">&times;</button>
  </div>

  <div class="dict-body">
    {#if loading}
      <p class="dict-status">Looking up...</p>
    {:else if error}
      <p class="dict-status">{error}</p>
    {:else}
      {#each entries as entry}
        {#each entry.meanings as meaning}
          <div class="dict-meaning">
            <span class="dict-pos">{meaning.partOfSpeech}</span>
            {#each meaning.definitions as def, i}
              <p class="dict-def">
                <span class="dict-num">{i + 1}.</span>
                {def.definition}
              </p>
              {#if def.example}
                <p class="dict-example">"{def.example}"</p>
              {/if}
            {/each}
          </div>
        {/each}
      {/each}
    {/if}
  </div>
  </div>
</div>

<style>
  .popover-overlay {
    position: fixed;
    inset: 0;
    z-index: 70;
  }

  .dict-popup {
    position: fixed;
    z-index: 71;
    background: var(--reader-panel-bg, #fff);
    color: var(--reader-panel-fg, #1a1a1a);
    border: 1px solid var(--reader-border, rgba(0, 0, 0, 0.15));
    border-radius: 12px;
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.2);
    max-height: 300px;
    display: flex;
    flex-direction: column;
    animation: pop 0.15s ease-out;
  }

  @keyframes pop {
    from { opacity: 0; transform: scale(0.95); }
    to   { opacity: 1; transform: scale(1); }
  }

  .dict-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 0.75rem;
    border-bottom: 1px solid var(--reader-border, rgba(0, 0, 0, 0.08));
    flex-shrink: 0;
  }

  .dict-word {
    font-size: 0.95rem;
    font-weight: 700;
  }

  .dict-phonetic {
    font-size: 0.8rem;
    color: var(--reader-muted, #9ca3af);
  }

  .dict-close {
    margin-left: auto;
    background: none;
    border: none;
    font-size: 1.1rem;
    cursor: pointer;
    color: var(--reader-muted, #9ca3af);
    padding: 0;
    line-height: 1;
  }
  .dict-close:hover {
    color: var(--reader-panel-fg, #1a1a1a);
  }

  .dict-body {
    overflow-y: auto;
    padding: 0.5rem 0.75rem 0.75rem;
  }

  .dict-status {
    color: var(--reader-muted, #9ca3af);
    font-size: 0.85rem;
    padding: 0.5rem 0;
  }

  .dict-meaning {
    margin-bottom: 0.5rem;
  }

  .dict-pos {
    font-size: 0.7rem;
    font-weight: 600;
    color: #4f46e5;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .dict-def {
    font-size: 0.85rem;
    margin: 0.3rem 0 0;
    line-height: 1.45;
  }

  .dict-num {
    color: var(--reader-muted, #9ca3af);
    font-weight: 600;
  }

  .dict-example {
    font-size: 0.8rem;
    color: var(--reader-muted, #9ca3af);
    font-style: italic;
    margin: 0.2rem 0 0 1rem;
  }
</style>
