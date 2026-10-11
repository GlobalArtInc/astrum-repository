import { API_INTERNAL_UI, ENDPOINTS, restClient } from '../../../../../../interface/api';
import { NotificationChannel, NotificationTestResult, RepositoryOption } from './types';

export const NOTIFICATIONS_URL = `${API_INTERNAL_UI}/notifications`;

const channelUrl = (id: string) => `${NOTIFICATIONS_URL}/${encodeURIComponent(id)}`;

export function listChannels(): Promise<NotificationChannel[]> {
  return restClient.get<NotificationChannel[]>(NOTIFICATIONS_URL);
}

export function createChannel(channel: NotificationChannel): Promise<NotificationChannel> {
  return restClient.post<NotificationChannel>(NOTIFICATIONS_URL, channel);
}

export function updateChannel(id: string, channel: NotificationChannel): Promise<NotificationChannel> {
  return restClient.put<NotificationChannel>(channelUrl(id), channel);
}

export function deleteChannel(id: string): Promise<void> {
  return restClient.delete(channelUrl(id));
}

export function testChannel(id: string): Promise<NotificationTestResult> {
  return restClient.post<NotificationTestResult>(`${channelUrl(id)}/test`);
}

export function testUnsavedChannel(channel: NotificationChannel): Promise<NotificationTestResult> {
  return restClient.post<NotificationTestResult>(`${NOTIFICATIONS_URL}/test`, channel);
}

export async function listRepositories(): Promise<RepositoryOption[]> {
  const repositories = await restClient.get<RepositoryOption[]>(ENDPOINTS.REPOSITORIES);
  return (repositories || [])
    .filter((repository) => repository.type !== 'group')
    .sort((a, b) => a.name.localeCompare(b.name));
}

/**
 * The field and message of an error answered by the notifications REST resource.
 */
export function serverError(error: unknown, fallback: string): { field?: string; message: string } {
  const data = (error as { response?: { data?: unknown } })?.response?.data;
  const first = Array.isArray(data) ? data[0] : data;
  if (first && typeof first === 'object' && typeof (first as { message?: unknown }).message === 'string') {
    const { id, message } = first as { id?: string; message: string };
    return { field: id && id !== '*' ? id : undefined, message };
  }
  return { message: fallback };
}
