import Component from '@glimmer/component';
import { action } from '@ember/object';
import { inject as service } from '@ember/service';

export default class JoypadComponent extends Component {
    @service serverConnection;
    @service gamepad;

    _gamepad = null;

    @action
    didInsert() {
        this._gamepad = this.gamepad.addGamepadEventListener(this.gamepadChanged.bind(this));
    }

    @action
    willDestroy() {
        if(this._gamepad !== null) {
            this.gamepad.removeGamepadEventListener(this._gamepad);
            this._gamepad = null;
        }
    }

    @action
    mousedown(key) {
        this.serverConnection.send({
            cmd: "emulator input",
            key,
            press: true,
        });
    }

    @action
    mouseup(key) {
        this.serverConnection.send({
            cmd: "emulator input",
            key,
            press: false,
        });
    }

    mapKey(code) {
        switch(code) {
            case "KeyW":
            case "ArrowUp":
                return "up";
            case "KeyS":
            case "ArrowDown":
                return "down";
            case "KeyA":
            case "ArrowLeft":
                return "left";
            case "KeyD":
            case "ArrowRight":
                return "right";
            case "KeyK":
                return "a";
            case "KeyL":
                return "b";
            case "Enter":
                return "start";
            case "Space":
                return "select";
        }
    }

    @action
    keydown(event) {
        let key = this.mapKey(event.code);
        if(key) {
            event.preventDefault();
            event.stopPropagation();
            if(event.repeat) {
                return;
            }
            this.serverConnection.send({
                cmd: "emulator input",
                key,
                press: true,
            });
        }
    }

    @action
    keyup(event) {
        let key = this.mapKey(event.code);
        if(key) {
            event.preventDefault();
            event.stopPropagation();
            this.serverConnection.send({
                cmd: "emulator input",
                key,
                press: false,
            });
        }
    }

    mapGamepadButton(buttonIdx) {
        switch(buttonIdx) {
            case 0:
            case 1:
                return "a";
            case 2:
            case 3:
                return "b";
            case 8:
                return "select";
            case 9:
                return "start";
            default:
                return null;
        }
    }

    sendGamepadAxis(oldValue, newValue, negative, positive) {
        let oldKey = null, newKey = null;
        if(oldValue < -0.5) {
            oldKey = negative;
        } else if(oldValue > 0.5) {
            oldKey = positive;
        }
        if(newValue < -0.5) {
            newKey = negative;
        } else if(newValue > 0.5) {
            newKey = positive;
        }
        if(oldKey !== newKey) {
            if(oldKey) {
                this.serverConnection.send({
                    cmd: "emulator input",
                    key: oldKey,
                    press: false,
                });
            }
            if(newKey) {
                this.serverConnection.send({
                    cmd: "emulator input",
                    key: newKey,
                    press: true,
                });
            }
        }
    }

    gamepadChanged(gamepad, oldButtons, newButtons, oldAxes, newAxes) {
        console.log('gamepad', gamepad, 'buttons', oldButtons, newButtons, 'axes', oldAxes, newAxes);
        for(let i = 0; i < oldButtons.length; ++i) {
            if(oldButtons[i] !== newButtons[i]) {
                const key = this.mapGamepadButton(i);
                if(key) {
                    this.serverConnection.send({
                        cmd: "emulator input",
                        key,
                        press: newButtons[i],
                    });
                }
            }
        }
        if(Math.abs(oldAxes[0] - newAxes[0]) > 0.01) {
            this.sendGamepadAxis(oldAxes[0], newAxes[0], "left", "right");
        }
        if(Math.abs(oldAxes[1] - newAxes[1]) > 0.01) {
            this.sendGamepadAxis(oldAxes[1], newAxes[1], "up", "down");
        }
    }
}
