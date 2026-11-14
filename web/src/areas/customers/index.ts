import type { Area } from '../../app/area';
import { CUSTOMERS_PAGE } from './api';
import { READ } from './model';

export const customersArea: Area = {
  id: 'customers',
  routes: [
    { path: 'admin/customers', load: () => import('./CustomersPage'), need: READ },
    { path: 'admin/customers/:id', load: () => import('./CustomerPage'), need: READ },
  ],
  settings: [{ id: 'customers', to: CUSTOMERS_PAGE, need: READ }],
};
