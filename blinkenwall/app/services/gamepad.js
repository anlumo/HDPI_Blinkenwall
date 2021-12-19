import Service from '@ember/service';

let id = 0;

export default class GamepadService extends Service {
    gamepads = new Map();

    _eventListeners = new Map();
    _currentAnimationFrame = null;

    init() {
        super.init();
        window.addEventListener("gamepadconnected", this.connected.bind(this));
        window.addEventListener("gamepaddisconnected", this.disconnected.bind(this));
    }

    addGamepadEventListener(fn) {
        this._eventListeners.set(id, fn);
        if(this.gamepads.size > 0 && this._currentAnimationFrame === null) {
            console.log('starting polling');
            this._currentAnimationFrame = window.requestAnimationFrame(this.pollGamepads.bind(this));
        }
        return id++;
    }

    removeGamepadEventListener(removeId) {
        this._eventListeners.delete(removeId);
        if(this._eventListeners.size === 0 && this._currentAnimationFrame !== null) {
            console.log('ending polling');
            window.cancelAnimationFrame(this._currentAnimationFrame);
            this._currentAnimationFrame = null;
        }
    }

    connected(event) {
        if(this.gamepads.size > 0) {
            return; // don't support more than one controller right now
        }
        this.gamepads.set(event.gamepad, {
            buttons: event.gamepad.buttons.map((button) => button.pressed),
            axes: event.gamepad.axes.slice(),
        });
        if(this._eventListeners.size > 0 && this._currentAnimationFrame === null) {
            console.log('starting polling');
            this._currentAnimationFrame = window.requestAnimationFrame(this.pollGamepads.bind(this));
        }
    }

    disconnected(event) {
        console.log('gamepad disconnected', event);
        this.gamepads.delete(event.gamepad);
        if(this.gamepads.size === 0 && this._currentAnimationFrame !== null) {
            console.log('ending polling');
            window.cancelAnimationFrame(this._currentAnimationFrame);
            this._currentAnimationFrame = null;
        }
    }

    pollGamepads() {
        const { gamepads } = this;
        const changed = [];
        for(const [gamepad, { buttons: oldState, axes: oldAxes }] of gamepads) {
            const newState = gamepad.buttons.map((button) => button.pressed);
            const newAxes = gamepad.axes;
            let isChanged = false;
            for(let i = 0; i < newAxes.length; ++i) {
                if(Math.abs(newAxes[i] - oldAxes[i]) > 0.01) {
                    changed.push({ gamepad, newState, newAxes: newAxes.slice() });
                    isChanged = true;
                    break;
                }
            }
            if(!isChanged) {
                for(let i = 0; i < newState.length; ++i) {
                    if(newState[i] !== oldState[i]) {
                        changed.push({ gamepad, newState, newAxes: newAxes.slice() });
                        break;
                    }
                }
            }
        }
        for (const change of changed) {
            const old = gamepads.get(change.gamepad);
            for (const listener of this._eventListeners.values()) {
                listener(change.gamepad, old.buttons, change.newState, old.axes, change.newAxes);
            }
            gamepads.set(change.gamepad, { buttons: change.newState, axes: change.newAxes });
        }

        this._currentAnimationFrame = window.requestAnimationFrame(this.pollGamepads.bind(this));
    }
}
