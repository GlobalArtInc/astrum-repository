
package org.globalart.astrum.internal.node;

import java.util.Optional;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

@RunWith(MockitoJUnitRunner.Silent.class)
public class DeploymentIdStoreImplTest
{
  private static final String DEPLOYMENT_ID = "ipuaeycw934r";

  private static final String DEPLOYMENT_ID_NEW = "ipuaeycw934re4rgt0f34r";

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(DeploymentIdDAO.class);

  @Mock
  private EventManager eventManager;

  private DeploymentIdStoreImpl underTest;

  @Before
  public void setup() {
    underTest = new DeploymentIdStoreImpl(sessionRule);
    underTest.setDependencies(eventManager);
  }

  @Test
  public void testGet_must_be_empty() {
    Optional<String> deploymentId = underTest.get();

    assertThat(deploymentId.isPresent(), is(false));
  }

  @Test
  public void testGet_must_contain_deployment_id() {
    underTest.set(DEPLOYMENT_ID);

    Optional<String> deploymentId = underTest.get();

    assertThat(deploymentId.isPresent(), is(true));
    assertThat(deploymentId.get(), is(DEPLOYMENT_ID));
  }

  @Test
  public void testGet_must_contain_new_deployment_id() {
    underTest.set(DEPLOYMENT_ID);

    Optional<String> deploymentId = underTest.get();

    assertThat(deploymentId.isPresent(), is(true));
    assertThat(deploymentId.get(), is(DEPLOYMENT_ID));

    underTest.set(DEPLOYMENT_ID_NEW);

    deploymentId = underTest.get();

    assertThat(deploymentId.isPresent(), is(true));
    assertThat(deploymentId.get(), is(DEPLOYMENT_ID_NEW));
  }
}
