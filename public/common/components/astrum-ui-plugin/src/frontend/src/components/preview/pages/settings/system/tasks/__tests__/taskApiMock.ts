

/**
 * Shared jest.mock factory for the internal API module (interface/api) consumed by
 * tasksFormMachine. Without it, the machine's restClient.get('/tasks/templates') call
 * fires a real network request that rejects with AxiosError: Network Error and emits
 * 'Failed to load task types' noise into the Jest output of the tasks suites.
 *
 * This is a partial payload: it includes only the fields fetchTaskTypes parses
 * (type/name/concurrentRun/properties), not the full TaskTemplateXO shape (which also
 * carries enabled/alertEmail/notificationCondition/frequency). (ASTRUM-52612)
 *
 * Unlike the empty-array mock in tasksFormMachine.test.ts, this resolves with realistic
 * records so the factory doubles as a reference for the parsed /tasks/templates contract.
 * The content is otherwise irrelevant here: TaskForm/TasksPage mock useTasksForm and
 * useTasksApi, so the machine's parsed context is never surfaced — only the resolution
 * (not its value) silences the load-error noise.
 *
 * Use via require() inside the jest.mock factory to satisfy jest's hoisting rule:
 *   jest.mock('.../interface/api', () => require('./taskApiMock').createTaskApiMock());
 */
export const createTaskApiMock = () => ({
  ENDPOINTS: {TASKS: '/service/rest/v1/tasks'},
  restClient: {
    get: jest.fn().mockResolvedValue([
      {type: 'repository.cleanup', name: 'Cleanup repositories', concurrentRun: false, properties: {}},
      {type: 'db.backup', name: 'Database backup', concurrentRun: false, properties: {}},
    ]),
  },
});
