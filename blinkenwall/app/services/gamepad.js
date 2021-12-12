import Service from '@ember/service';

let id = 0;

export default class GamepadService extends Service {
    gamepads = new Map();

    _eventListeners = new Map();
    _currentAnimationFrame = null;

    init() {
        this._super();
        window.addEventListener("gamepadconnected", this.connected.bind(this));
        window.addEventListener("gamepaddisconnected", this.disconnected.bind(this));
    }

    addGamepadEventListener(fn) {
        this._eventListeners.set(id, fn);
        if(this.gamepads.size > 0 && this._currentAnimationFrame === null) {
            this._currentAnimationFrame = window.requestAnimationFrame(this.pollGamepads.bind(this));
        }
        return id++;
    }

    removeGamepadEventListener(removeId) {
        this._eventListeners.delete(removeId);
        if(this._eventListeners.length === 0 && this._currentAnimationFrame !== null) {
            window.cancelAnimationFrame(this._currentAnimationFrame);
            this._currentAnimationFrame = null;
        }
    }

    connected(event) {
        this.gamepads.set(event.gamepad, event.gamepad.buttons.map((button) => button.pressed));
        if(this._eventListeners.length > 0 && this._currentAnimationFrame === null) {
            this._currentAnimationFrame = window.requestAnimationFrame(this.pollGamepads.bind(this));
        }
    }

    disconnected(event) {
        this.gamepads.delete(event.gamepad);
        if(this.gamepads.size === 0 && this._currentAnimationFrame !== null) {
            window.cancelAnimationFrame(this._currentAnimationFrame);
            this._currentAnimationFrame = null;
        }
    }

    pollGamepads() {
        const { gamepads } = this;
        const changed = [];
        for(const [gamepad, oldState] in gamepads) {
            const newState = gamepad.buttons.map((button) => button.pressed);
            for(const i = 0; i < newState.length; ++i) {
                if(newState[i] !== oldState[i]) {
                    changed.push({ gamepad, newState });
                    break;
                }
            }
        }
        for (const change in changed) {
            gamepads.set(change.gamepad, change.newState);
            for (const listener in this._eventListeners) {
                listener.fn(change.gamepad, change.newState);
            }
        }

        this._currentAnimationFrame = window.requestAnimationFrame(this.pollGamepads.bind(this));
    }
}
