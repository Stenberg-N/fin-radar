export class Gutter {
  #timeout: ReturnType<typeof setTimeout> | null = null;
  #isMoving = $state(false);
  #isHovered = $state(false);

  get isMoving() {
    return this.#isMoving;
  };

  get isHovered() {
    return this.#isHovered;
  };

  enter = () => {
    if (this.#timeout) clearTimeout(this.#timeout);
    this.#timeout = null;

    this.#timeout = setTimeout(() => {
      this.#isHovered = true;
    }, 500);
  };

  leave = () => {
    if (this.#timeout) clearTimeout(this.#timeout);
    this.#timeout = null;
    this.#isHovered = false;
  };

  destroy = () => {
    if (this.#timeout) clearTimeout(this.#timeout)
  };
}