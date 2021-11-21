import Route from '@ember/routing/route';
import { inject as service } from '@ember/service';

export default class EmulatorRoute extends Route {
  @service serverConnection;

  async model() {
    const response = await new Promise((resolve) => {
      this.serverConnection.send(
        {
          cmd: 'emulator list',
        },
        resolve
      );
    });
    return { roms: response.roms };
  }
}
