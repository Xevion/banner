import { type Component, type Snippet, mount, unmount } from "svelte";

/**
 * Wraps a Svelte component so TanStack Table can render it as a column
 * header or cell. Returns a `RenderComponentConfig` that `FlexRender`
 * picks up.
 */
export function renderComponent<
  TProps extends Record<string, unknown>,
  TComp extends Component<TProps>,
>(component: TComp, props: TProps) {
  return {
    component,
    props,
    [RENDER_COMPONENT_SYMBOL]: true,
  } as const;
}

/**
 * Wraps a Svelte 5 raw snippet for use in TanStack Table column defs.
 */
export function renderSnippet<TProps>(snippet: Snippet<[TProps]>, props: TProps) {
  return {
    snippet,
    props,
    [RENDER_SNIPPET_SYMBOL]: true,
  } as const;
}

// Symbols for FlexRender to detect render types
export const RENDER_COMPONENT_SYMBOL = Symbol("renderComponent");
export const RENDER_SNIPPET_SYMBOL = Symbol("renderSnippet");

export interface RenderComponentConfig<
  TProps extends Record<string, unknown> = Record<string, unknown>,
> {
  component: Component<TProps>;
  props: TProps;
  [RENDER_COMPONENT_SYMBOL]: true;
}

export interface RenderSnippetConfig<TProps = unknown> {
  snippet: Snippet<[TProps]>;
  props: TProps;
  [RENDER_SNIPPET_SYMBOL]: true;
}

export function isRenderComponentConfig(value: unknown): value is RenderComponentConfig {
  return typeof value === "object" && value !== null && RENDER_COMPONENT_SYMBOL in value;
}

export function isRenderSnippetConfig(value: unknown): value is RenderSnippetConfig {
  return typeof value === "object" && value !== null && RENDER_SNIPPET_SYMBOL in value;
}

/**
 * Mount a Svelte component imperatively into a target element.
 * Used by FlexRender for component-type cells.
 */
export function mountComponent<TProps extends Record<string, unknown>>(
  component: Component<TProps>,
  target: HTMLElement,
  props: TProps
): () => void {
  const instance = mount(component, { target, props });
  return () => {
    void unmount(instance);
  };
}
