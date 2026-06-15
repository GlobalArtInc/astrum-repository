
import { ROUTE_NAMES } from '../routeNames/routeNames';
import SelfHostedLoginPageWrapper from '../../components/login/SelfHostedLoginPageWrapper';

export const loginRoutes = [
  {
    name: ROUTE_NAMES.LOGIN,
    url: '/login?returnTo',
    component: SelfHostedLoginPageWrapper,
    params: {
      returnTo: {
        type: 'string',
        value: null,
        squash: true
      }
    },
    data: {
      visibilityRequirements: {},
      title: 'Login',
    }
  }
];
