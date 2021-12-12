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

    gamepadChanged(gamepad, buttons) {
        // TODO
    }
}
