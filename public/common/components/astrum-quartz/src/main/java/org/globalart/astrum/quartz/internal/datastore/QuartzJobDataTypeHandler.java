
package org.globalart.astrum.quartz.internal.datastore;

import java.io.IOException;
import java.io.InputStream;
import java.io.ObjectInputStream;
import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.mybatis.AbstractBytesTypeHandler;

import org.quartz.JobDataMap;
import org.springframework.stereotype.Component;

/*
 * Simplified from org.quartz.impl.jdbcjobstore.StdJDBCDelegate as Astrum configures Quartz for Property style
 * serialization rather than ObjectInputStream
 *
 * See also org.quartz.impl.jdbcjobstore.PostgreSQLDelegate
 */
@Component
public class QuartzJobDataTypeHandler
    extends AbstractBytesTypeHandler<JobDataMap>
{
  @Autowired
  public QuartzJobDataTypeHandler() {
    super(JobDataMap::new);
  }

  @Override
  protected JobDataMap deserialize(final InputStream in) {
    try (ObjectInputStream ois = new ObjectInputStream(in)) {
      return new JobDataMap((Map<?, ?>) ois.readObject());
    }
    catch (IOException e) {
      log.debug("An unepected error occurred", e);
      return null;
    }
    catch (ClassNotFoundException e) {
      log.debug("An unepected error occurred", e);
      return null;
    }
  }
}
