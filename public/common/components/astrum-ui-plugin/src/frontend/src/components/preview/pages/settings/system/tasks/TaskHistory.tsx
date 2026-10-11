

import React from 'react';
import { Box, Flex, Text, Badge } from '@radix-ui/themes';
import { Clock, CheckCircle, XCircle, History } from 'lucide-react';

import { TaskHistoryProps, formatDate } from './types';
import { getTaskStatusLabel } from './taskLabels';

import './TaskHistory.scss';
import TasksStrings from '../../../../constants/pages/settings/system/TasksStrings';

/**
 * TaskHistory - Displays task execution history and last run information
 * 
 * Note: The current API only provides last run information.
 * A full execution history would require a dedicated API endpoint.
 */
export function TaskHistory({ task }: TaskHistoryProps) {
  const hasLastRun = Boolean(task.lastRun);
  const isSuccess = task.lastRunResult?.toLowerCase().startsWith('ok');
  const isFailed = task.lastRunResult?.toLowerCase().includes('error') || 
                   task.lastRunResult?.toLowerCase().includes('fail');
  const isCanceled = task.lastRunResult?.toLowerCase().includes('cancel');

  const getResultIcon = () => {
    if (isSuccess) return <CheckCircle size={16} className="task-history__icon task-history__icon--success" />;
    if (isFailed) return <XCircle size={16} className="task-history__icon task-history__icon--error" />;
    if (isCanceled) return <XCircle size={16} className="task-history__icon task-history__icon--canceled" />;
    return <Clock size={16} className="task-history__icon" />;
  };

  const getResultColor = (): 'green' | 'red' | 'yellow' | 'gray' => {
    if (isSuccess) return 'green';
    if (isFailed) return 'red';
    if (isCanceled) return 'yellow';
    return 'gray';
  };

  return (
    <Box className="task-history">
      <Flex align="center" gap="2" className="task-history__header">
        <History size={18} />
        <Text size="3" weight="medium">{TasksStrings.HISTORY.EXECUTION_HISTORY}</Text>
      </Flex>

      <Box className="task-history__content">
        {hasLastRun ? (
          <Box className="task-history__run" data-testid="task-history-last-run">
            <Text size="3" weight="medium" mb="3">{TasksStrings.HISTORY.LAST_EXECUTION}</Text>
            <Flex direction="column" gap="3">
              <Flex justify="between" align="start" gap="4">
                <Box>
                  <Text as="div" size="1" color="gray">{TasksStrings.HISTORY.STARTED}</Text>
                  <Text size="2">{formatDate(task.lastRun)}</Text>
                </Box>
                <Box>
                  <Text as="div" size="1" color="gray">{TasksStrings.HISTORY.RESULT}</Text>
                  <Flex align="center" gap="2">
                    {getResultIcon()}
                    <Badge color={getResultColor()} size="1">
                      {task.lastRunResult ? getTaskStatusLabel(task.lastRunResult) : TasksStrings.HISTORY.UNKNOWN}
                    </Badge>
                  </Flex>
                </Box>
              </Flex>
              {task.statusDescription && (
                <Box>
                  <Text as="div" size="1" color="gray">{TasksStrings.HISTORY.DETAILS}</Text>
                  <Text size="2">{task.statusDescription}</Text>
                </Box>
              )}
            </Flex>
          </Box>
        ) : (
          <Box className="task-history__empty" data-testid="task-history-empty">
            <Flex direction="column" align="center" gap="2" py="4">
              <Clock size={24} className="task-history__icon" />
              <Text size="2" weight="medium">{TasksStrings.HISTORY.NO_EXECUTION_HISTORY}</Text>
              <Text size="1" color="gray">{TasksStrings.HISTORY.TASK_HAS_NOT_BEEN_EXECUTED}</Text>
            </Flex>
          </Box>
        )}

        {/* Next run information */}
        {task.nextRun && (
          <Box className="task-history__next">
            <Text size="2" weight="medium" className="task-history__label">{TasksStrings.HISTORY.NEXT_SCHEDULED_RUN}</Text>
            <Flex align="center" gap="2">
              <Clock size={14} className="task-history__icon task-history__icon--next" />
              <Text size="2" className="task-history__value">
                {formatDate(task.nextRun)}
              </Text>
            </Flex>
          </Box>
        )}

        {/* Schedule info */}
        <Box className="task-history__schedule">
          <Text size="2" weight="medium" className="task-history__label">{TasksStrings.HISTORY.SCHEDULE}</Text>
          <Text size="2" className="task-history__value task-history__value--schedule">
            {task.schedule === 'manual' ? TasksStrings.HISTORY.MANUAL_RUN_DEMAND :
             task.schedule === 'once' ? TasksStrings.HISTORY.ONCE :
             task.schedule === 'hourly' ? TasksStrings.HISTORY.HOURLY :
             task.schedule === 'daily' ? TasksStrings.HISTORY.DAILY :
             task.schedule === 'weekly' ? TasksStrings.HISTORY.WEEKLY :
             task.schedule === 'monthly' ? TasksStrings.HISTORY.MONTHLY :
             task.schedule === 'advanced' ? TasksStrings.HISTORY.CRON(task.cronExpression) :
             task.schedule}
          </Text>
        </Box>
      </Box>
    </Box>
  );
}

export default TaskHistory;


