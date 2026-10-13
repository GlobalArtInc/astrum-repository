import { screen, within } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import { TasksTab, tasksOf } from './EditTabs';

type Task = Parameters<typeof tasksOf>[0][number];

const TASKS: Task[] = [
  {
    id: 'a1',
    name: 'Purge maven-central',
    typeName: 'Repository - Delete unused components',
    enabled: true,
    nextRun: '2026-10-14T01:00:00.000Z',
    properties: { repositoryName: 'Maven-Central', lastUsed: '30' },
  },
  {
    id: 'a2',
    name: 'Purge everything',
    typeName: 'Repository - Delete unused components',
    enabled: true,
    nextRun: null,
    properties: { repositoryName: '*' },
  },
  {
    id: 'a3',
    name: 'Compact',
    typeName: 'Admin - Compact blob store',
    enabled: true,
    nextRun: null,
    properties: { blobstoreName: 'default' },
  },
];

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the tasks of a repository', () => {
  it('are the tasks that name the repository', () => {
    expect(tasksOf(TASKS, 'maven-central').map((task) => task.id)).toEqual(['a1']);
    expect(tasksOf(TASKS, 'npmjs')).toEqual([]);
  });

  it('are listed with links to their pages', async () => {
    const server = fakeServer({ 'GET /service/rest/internal/ui/tasks': () => jsonResponse(TASKS) });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<TasksTab repository="maven-central" />);
    const card = await screen.findByRole('region', { name: 'Scheduled Tasks' });
    expect(await within(card).findByRole('link', { name: 'Purge maven-central' })).toHaveAttribute(
      'href',
      '/admin/system/tasks/a1',
    );
    expect(within(card).getByText('1 task')).toBeInTheDocument();
    expect(within(card).queryByText('Purge everything')).not.toBeInTheDocument();
  });
});
