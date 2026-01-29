<script lang="ts" module>
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type FlexRenderProps<TProps = any> = {
  content: unknown;
  context: TProps;
};
</script>

<script lang="ts">
  import { isRenderComponentConfig, isRenderSnippetConfig, mountComponent } from "./render-helpers.js";

  let { content, context }: FlexRenderProps = $props();

  function renderAction(node: HTMLElement, contentVal: typeof content) {
    let cleanup: (() => void) | undefined;

    function render(c: typeof content) {
      cleanup?.();
      node.textContent = "";

      if (isRenderComponentConfig(c)) {
        cleanup = mountComponent(c.component, node, c.props as Record<string, unknown>);
      }
    }

    render(contentVal);

    return {
      update(newContent: typeof content) {
        render(newContent);
      },
      destroy() {
        cleanup?.();
      },
    };
  }
</script>

{#if isRenderSnippetConfig(content)}
  {@render content.snippet(content.props)}
{:else if isRenderComponentConfig(content)}
  <div use:renderAction={content}></div>
{:else if typeof content === "function"}
  {@const result = content(context)}
  {#if isRenderComponentConfig(result)}
    <div use:renderAction={result}></div>
  {:else if isRenderSnippetConfig(result)}
    {@render result.snippet(result.props)}
  {:else if typeof result === "string" || typeof result === "number"}
    {result}
  {/if}
{:else if typeof content === "string" || typeof content === "number"}
  {content}
{/if}
