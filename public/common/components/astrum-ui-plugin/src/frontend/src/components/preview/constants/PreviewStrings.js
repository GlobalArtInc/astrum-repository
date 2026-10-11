/**
 * The strings modules of Astrum One. Components import the module of their page directly; this list is what
 * tools/i18n reads the English texts from and what the tests of the translations load.
 */
export { default as CommonStrings } from './CommonStrings';
export { default as TasksStrings } from './pages/settings/system/TasksStrings';
export { TASK_TYPE_SELECTOR, DYNAMIC_FORM_FIELDS } from '../pages/settings/system/tasks/TaskStrings';
