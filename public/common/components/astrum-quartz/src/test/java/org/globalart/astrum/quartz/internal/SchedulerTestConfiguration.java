
package org.globalart.astrum.quartz.internal;

import java.util.List;
import java.util.concurrent.CyclicBarrier;
import java.util.concurrent.TimeUnit;

import org.quartz.Job;
import org.quartz.JobExecutionContext;
import org.quartz.JobExecutionException;
import org.quartz.SchedulerContext;
import org.springframework.boot.test.context.TestConfiguration;
import org.springframework.context.annotation.Bean;

/**
 * Test configuration for SchedulerTest that provides the test job classes.
 */
@TestConfiguration
public class SchedulerTestConfiguration
{
  private static final String BARRIER = "BARRIER";

  private static final String DATE_STAMPS = "DATE_STAMPS";

  private static final String JOB_THREAD = "JOB_THREAD";

  public static final long TEST_TIMEOUT_SECONDS = 125;

  public static class TestJob
      implements Job
  {
    public void execute(JobExecutionContext context) throws JobExecutionException {
      // nop
    }
  }

  public static class TestJobWithSync
      implements Job
  {
    public void execute(JobExecutionContext context) throws JobExecutionException {
      try {
        @SuppressWarnings("unchecked")
        List<Long> jobExecTimestamps = (List<Long>) context.getScheduler().getContext().get(DATE_STAMPS);
        CyclicBarrier barrier = (CyclicBarrier) context.getScheduler().getContext().get(BARRIER);

        jobExecTimestamps.add(System.currentTimeMillis());

        barrier.await(TEST_TIMEOUT_SECONDS, TimeUnit.SECONDS);
      }
      catch (Throwable e) {
        e.printStackTrace();
        throw new AssertionError("Await on barrier was interrupted: " + e.toString());
      }
    }
  }

  public static class UncleanShutdownJob
      implements Job
  {
    @Override
    public void execute(JobExecutionContext context) throws JobExecutionException {
      try {
        SchedulerContext schedulerContext = context.getScheduler().getContext();
        schedulerContext.put(JOB_THREAD, Thread.currentThread());
        CyclicBarrier barrier = (CyclicBarrier) schedulerContext.get(BARRIER);
        barrier.await(TEST_TIMEOUT_SECONDS, TimeUnit.SECONDS);
      }
      catch (Throwable e) {
        e.printStackTrace();
        throw new AssertionError("Await on barrier was interrupted: " + e.toString());
      }
    }
  }

  @Bean
  public TestJob testJob() {
    return new TestJob();
  }

  @Bean
  public TestJobWithSync testJobWithSync() {
    return new TestJobWithSync();
  }

  @Bean
  public UncleanShutdownJob uncleanShutdownJob() {
    return new UncleanShutdownJob();
  }
}
