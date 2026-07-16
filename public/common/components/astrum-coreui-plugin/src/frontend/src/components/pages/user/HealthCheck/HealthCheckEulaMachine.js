
import {Machine} from "xstate";
import Axios from "axios";

// EULA acceptance endpoint for Health Check features
const healthCheckEulaUrl = 'service/rest/internal/ui/healthcheck/eula';

export default Machine(
    {
      id: 'HealthCheckEulaMachine',
      initial: 'idle',

      states: {
        idle: {
          on: {
            ACCEPT: {
              target: 'accept'
            },
            DECLINE: {
              actions: ['eulaDeclined']
            }
          }
        },
        accept: {
          invoke: {
            src: 'acceptEula',
            onDone: {
              actions: ['eulaAccepted']
            }
          }
        },
      }
    },
    {
      services: {
        acceptEula: () => Axios.put(healthCheckEulaUrl)
      }
    });
