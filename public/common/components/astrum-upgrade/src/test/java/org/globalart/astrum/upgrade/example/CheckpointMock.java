
package org.globalart.astrum.upgrade.example;

import org.globalart.astrum.common.upgrade.Checkpoint;

import static org.mockito.Mockito.mock;

public abstract class CheckpointMock
    implements Checkpoint
{
  public final Checkpoint mock = mock(Checkpoint.class);

  @Override
  public void begin(String version) throws Exception {
    mock.begin(version);
  }

  @Override
  public void commit() throws Exception {
    mock.commit();
  }

  @Override
  public void rollback() throws Exception {
    mock.rollback();
  }

  @Override
  public void end() {
    mock.end();
  }
}
