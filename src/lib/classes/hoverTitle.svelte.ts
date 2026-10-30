export class HoverTitle<T = undefined> {
  isHovering = $state(false);
  /** Used to hold title content. */
  target = $state<T | undefined>(undefined);
  /** The reference for the hover title's timeout. */
  #timeout: ReturnType<typeof setTimeout> | null = null;
  /** The time after which the hover title is automatically killed. */
  #killTimeout: ReturnType<typeof setTimeout> | null = null;
  /** The time for which after the hover title spawns. */
  #delay: number;
  /** The identifier for finding and attaching a mouseleave listener on the correct hover title. */
  #ignoreSelector: string;
  /** The hover title. */
  #watched: Element | null = null;
  /** The element that triggered the `enter` method for the hover title. */
  #trigger: Element | null = null;
  static #count = 0;
  readonly id = `hover-${HoverTitle.#count++}`;

  constructor(delay = 500) {
    this.#delay = delay;
    this.#ignoreSelector = `[hover-title-owner="${this.id}"]`;
  }

  #unwatch = () => {
    this.#trigger?.removeEventListener('mousedown', this.#onLeave);
    this.#trigger?.removeEventListener('click', this.#onLeave);
    this.#trigger = null;
    this.#watched?.removeEventListener('mouseleave', this.#onLeave);
    this.#watched = null;
  };

  #hide = () => {
    if (this.#timeout) clearTimeout(this.#timeout);
    if (this.#killTimeout) clearTimeout(this.#killTimeout);
    this.#timeout = null;
    this.#killTimeout = null;
    this.isHovering = false;
  };

  #onLeave = () => {
    this.#unwatch();
    this.#hide();
  };

  enter = (e: MouseEvent, target?: T, autokill: boolean = true) => {
    this.#unwatch();
    if (this.#timeout) clearTimeout(this.#timeout);
    this.target = target;

    const trigger = e.currentTarget;
    if (trigger instanceof Element && autokill === true) {
      this.#trigger = trigger;
      trigger.addEventListener('mousedown', this.#onLeave);
      trigger.addEventListener('click', this.#onLeave);
    }

    this.#timeout = setTimeout(() => {
      if (this.#trigger && !this.#trigger.isConnected) return this.#onLeave();
      if (this.#trigger && !this.#trigger.matches(':hover')) return this.#onLeave();
      this.isHovering = true;
    }, this.#delay);
    if (autokill) this.#killTimeout = setTimeout(() => this.#onLeave(), 2500);
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
    if (this.#killTimeout) clearTimeout(this.#killTimeout);
  };
}