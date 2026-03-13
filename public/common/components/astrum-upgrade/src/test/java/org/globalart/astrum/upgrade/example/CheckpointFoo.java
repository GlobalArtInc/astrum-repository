
package org.globalart.astrum.upgrade.example;

import org.globalart.astrum.common.upgrade.Checkpoints;

@Checkpoints(model = "foo", local = true)
public class CheckpointFoo
    extends CheckpointMock
{
}
