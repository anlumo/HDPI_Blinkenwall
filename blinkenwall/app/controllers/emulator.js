import Controller from '@ember/controller';
import { action } from '@ember/object';
import { inject as service } from '@ember/service';

export default class EmulatorController extends Controller {
  @service serverConnection;

  @action
  startEmulator(rom) {
    this.serverConnection.send({
      cmd: 'emulator start',
      rom,
    });
  }
}
