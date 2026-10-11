export class HoverTitle<T = undefined> {
  isHovering = $state(false);
  target = $state<T | undefined>(undefined);
  #timeout: ReturnType<typeof setTimeout> | null = null;
  #delay: number;
  #ignoreSelector: string;
  #watched: Element | null = null;
  static #count = 0;
  readonly id = `hover-${HoverTitle.#count++}`;

  constructor(delay = 500) {
    this.#delay = delay;
    this.#ignoreSelector = `[hover-title-owner="${this.id}"]`;
  }

  #unwatch = () => {
    this.#watched?.removeEventListener('mouseleave', this.#onLeave);
    this.#watched = null;
  };

  #hide = () => {
    if (this.#timeout) clearTimeout(this.#timeout);
    this.#timeout = null;
    this.isHovering = false;
  };

  #onLeave = () => {
    this.#unwatch();
    this.#hide();
  };

  enter = (target?: T) => {
    this.#unwatch();
    if (this.#timeout) clearTimeout(this.#timeout);
    this.target = target;

    this.#timeout = setTimeout(() => this.isHovering = true, this.#delay);
  };

  leave = (e: MouseEvent) => {
    const next = e.relatedTarget;
    const trigger = e.currentTarget;
    const modal = next instanceof Element ? next.closest(this.#ignoreSelector) : null;
    const isOwnModal = modal && !(trigger instanceof Node && modal.contains(trigger));

    if (isOwnModal) {
      this.#unwatch();
      this.#watched = modal;
      modal.addEventListener('mouseleave', this.#onLeave);
      return;
    }
    
    this.#onLeave();
  };

  destroy = () => {
    this.#unwatch();
    if (this.#timeout) clearTimeout(this.#timeout);
  };
}