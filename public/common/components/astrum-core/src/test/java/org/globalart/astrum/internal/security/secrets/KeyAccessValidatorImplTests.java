
package org.globalart.astrum.internal.security.secrets;

import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.Collection;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.common.time.Clock;
import org.globalart.astrum.crypto.secrets.EncryptionKeyValidator;
import org.globalart.astrum.crypto.secrets.ReportKnownSecretKeyEvent;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.AstrumKeyValue;
import org.globalart.astrum.kv.ValueType;
import org.globalart.astrum.node.datastore.NodeHeartbeat;
import org.globalart.astrum.node.datastore.NodeHeartbeatManager;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasEntry;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class KeyAccessValidatorImplTests
{
  private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper();

  private static final String NODE_A = UUID.randomUUID().toString();

  private static final String NODE_B = UUID.randomUUID().toString();

  @Mock
  private EncryptionKeyValidator encryptionKeyValidator;

  @Mock
  private NodeHeartbeatManager nodeHeartbeatManager;

  @Mock
  private EventManager eventManager;

  @Mock
  private NodeAccess nodeAccess;

  @Mock
  private GlobalKeyValueStore globalKeyValueStore;

  @Mock
  private PeriodicJobService periodicJobService;

  @Mock
  private Clock clock;

  @Captor
  private ArgumentCaptor<AstrumKeyValue> keyValueCaptor;

  @Captor
  private ArgumentCaptor<ReportKnownSecretKeyEvent> eventCaptor;

  private KeyAccessValidatorImpl underTest;

  @Before
  public void setup() {
    when(nodeAccess.isClustered()).thenReturn(true);
    underTest = new KeyAccessValidatorImpl(encryptionKeyValidator, nodeHeartbeatManager, periodicJobService,
        eventManager, OBJECT_MAPPER, nodeAccess, globalKeyValueStore, clock, 1);
  }

  @Test
  public void testStoreIsWrittenOnReEncryptEvent() {
    when(nodeAccess.getId()).thenReturn(NODE_A);
    OffsetDateTime now = OffsetDateTime.now();
    when(clock.clusterTime()).thenReturn(now);
    when(encryptionKeyValidator.isValidKey("test-key")).thenReturn(true);

    ReportKnownSecretKeyEvent event = new ReportKnownSecretKeyEvent("test-key");
    underTest.on(event);
    verify(globalKeyValueStore).setKey(keyValueCaptor.capture());
    verify(periodicJobService).runOnce(any(), anyInt());
    AstrumKeyValue keyValue = keyValueCaptor.getValue();

    assertEquals(getNodeKey(NODE_A), keyValue.key());
    assertEquals(ValueType.OBJECT, keyValue.type());
    Map<String, Object> storedValue =
        keyValue.getAsObject(OBJECT_MAPPER, new TypeReference<HashMap<String, Object>>()
        {
        });
    assertThat(storedValue, hasEntry("keyId", "test-key"));
    assertThat(storedValue, hasEntry("hasAccess", true));
    assertThat(storedValue, hasEntry("timestamp", now.toString()));
  }

  @Test
  public void testActiveNodesNonCluster() {
    when(nodeAccess.isClustered()).thenReturn(false);
    when(nodeAccess.getId()).thenReturn(NODE_A);

    Set<String> activeNode = underTest.getActiveNodeIds();
    assertEquals(1, activeNode.size());
    verify(nodeAccess).isClustered();
    verify(nodeAccess).getId();
    verifyNoInteractions(nodeHeartbeatManager);
  }

  @Test
  public void testInvalidKeyOnTimeout() {
    OffsetDateTime now = OffsetDateTime.now();
    when(clock.clusterTime()).thenReturn(now);

    Collection<NodeHeartbeat> nodes = new ArrayList<>();
    nodes.add(createHeartbeatData(now, NODE_A));
    nodes.add(createHeartbeatData(now, NODE_B));
    when(nodeHeartbeatManager.getActiveNodeHeartbeatData()).thenReturn(nodes);

    // Nodes did not send the access data
    when(globalKeyValueStore.getKey(getNodeKey(NODE_A))).thenReturn(Optional.empty());

    assertFalse(underTest.isValidKey("test-key"));
    verify(eventManager).post(eventCaptor.capture());
    ReportKnownSecretKeyEvent postedEvent = eventCaptor.getValue();
    assertEquals("test-key", postedEvent.getKeyId());
  }

  @Test
  public void testInvalidKeyOnMissingAccessData() {
    OffsetDateTime now = OffsetDateTime.now();
    when(clock.clusterTime()).thenReturn(now);

    Collection<NodeHeartbeat> nodes = new ArrayList<>();
    nodes.add(createHeartbeatData(now, NODE_A));
    nodes.add(createHeartbeatData(now, NODE_B));
    when(nodeHeartbeatManager.getActiveNodeHeartbeatData()).thenReturn(nodes);

    // Only NODE_A has the access data
    when(globalKeyValueStore.getKey(getNodeKey(NODE_A)))
        .thenReturn(getAccessKeyInfo(NODE_A, "test-key", true, OffsetDateTime.now()));

    assertFalse(underTest.isValidKey("test-key"));
    verify(eventManager).post(eventCaptor.capture());
    ReportKnownSecretKeyEvent postedEvent = eventCaptor.getValue();
    assertEquals("test-key", postedEvent.getKeyId());
  }

  @Test
  public void testInvalidKeyOnNodeWithoutKeyAccess() {
    OffsetDateTime now = OffsetDateTime.now();
    when(clock.clusterTime()).thenReturn(now);

    Collection<NodeHeartbeat> nodes = new ArrayList<>();
    nodes.add(createHeartbeatData(now, NODE_A));
    nodes.add(createHeartbeatData(now, NODE_B));
    when(nodeHeartbeatManager.getActiveNodeHeartbeatData()).thenReturn(nodes);

    when(globalKeyValueStore.getKey(getNodeKey(NODE_A)))
        .thenReturn(getAccessKeyInfo(NODE_A, "test-key", true, OffsetDateTime.now()));
    // NODE_B does not have key access
    when(globalKeyValueStore.getKey(getNodeKey(NODE_B)))
        .thenReturn(getAccessKeyInfo(NODE_B, "test-key", false, OffsetDateTime.now()));

    assertFalse(underTest.isValidKey("test-key"));
    verify(eventManager).post(eventCaptor.capture());
    ReportKnownSecretKeyEvent postedEvent = eventCaptor.getValue();
    assertEquals("test-key", postedEvent.getKeyId());
  }

  @Test
  public void testInvalidKeyNodeWithoutPastHeartBeat() {
    OffsetDateTime now = OffsetDateTime.now();
    when(clock.clusterTime()).thenReturn(now);

    Collection<NodeHeartbeat> nodes = new ArrayList<>();
    nodes.add(createHeartbeatData(now, NODE_A));
    nodes.add(createHeartbeatData(now, NODE_B));
    when(nodeHeartbeatManager.getActiveNodeHeartbeatData()).thenReturn(nodes);

    when(globalKeyValueStore.getKey(getNodeKey(NODE_A)))
        .thenReturn(getAccessKeyInfo(NODE_A, "test-key", true, now.minusNanos(100)));
    when(globalKeyValueStore.getKey(getNodeKey(NODE_B)))
        .thenReturn(getAccessKeyInfo(NODE_B, "test-key", true, now.minusNanos(50)));

    assertFalse(underTest.isValidKey("test-key"));
    verify(eventManager).post(eventCaptor.capture());
    ReportKnownSecretKeyEvent postedEvent = eventCaptor.getValue();
    assertEquals("test-key", postedEvent.getKeyId());
  }

  @Test
  public void testValidKey() {
    String keyId = "validKey";

    OffsetDateTime now = OffsetDateTime.now();
    when(clock.clusterTime()).thenReturn(now);

    Collection<NodeHeartbeat> nodes = new ArrayList<>();
    nodes.add(createHeartbeatData(now, NODE_A));
    nodes.add(createHeartbeatData(now, NODE_B));
    when(nodeHeartbeatManager.getActiveNodeHeartbeatData()).thenReturn(nodes);

    when(globalKeyValueStore.getKey(getNodeKey(NODE_A)))
        .thenReturn(getAccessKeyInfo(NODE_A, keyId, true, OffsetDateTime.now()));
    when(globalKeyValueStore.getKey(getNodeKey(NODE_B)))
        .thenReturn(getAccessKeyInfo(NODE_B, keyId, true, OffsetDateTime.now()));

    assertTrue(underTest.isValidKey(keyId));
    verify(eventManager).post(eventCaptor.capture());

    ReportKnownSecretKeyEvent reportEvent = eventCaptor.getValue();
    assertEquals(keyId, reportEvent.getKeyId());
  }

  private NodeHeartbeat createHeartbeatData(final OffsetDateTime heartbeatTime, final String nodeId) {
    NodeHeartbeat nodeData = mock(NodeHeartbeat.class);

    Map<String, Object> nodeInfo = new HashMap<>();
    nodeInfo.put(NodeHeartbeatManager.NODE_ID, nodeId);

    when(nodeData.nodeInfo()).thenReturn(nodeInfo);
    when(nodeData.heartbeatTime()).thenReturn(heartbeatTime);
    return nodeData;
  }

  private Optional<AstrumKeyValue> getAccessKeyInfo(
      final String nodeId,
      final String keyId,
      final boolean hasAccess,
      final OffsetDateTime timestamp)
  {
    Map<String, Object> accessMap = new HashMap<>();
    accessMap.put("keyId", keyId);
    accessMap.put("hasAccess", hasAccess);
    accessMap.put("timestamp", timestamp.toString());

    return Optional.of(new AstrumKeyValue(getNodeKey(nodeId), ValueType.OBJECT, accessMap));
  }

  private String getNodeKey(final String nodeId) {
    return "re-encrypt.key.access." + nodeId;
  }
}
