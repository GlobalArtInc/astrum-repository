
package org.globalart.astrum.api.rest.selfhosted.blobstore;

import java.util.HashMap;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;

import jakarta.ws.rs.WebApplicationException;

import org.globalart.astrum.api.rest.common.blobstore.model.BlobStoreConnectionXO;
import org.globalart.astrum.api.rest.common.blobstore.model.BlobStoreQuotaResultXO;
import org.globalart.astrum.blobstore.ConnectionChecker;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConnectionException;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaResult;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;
import org.globalart.astrum.repository.blobstore.BlobStoreConfigurationStore;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentMatchers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class BlobStoreResourceTest
{
  @Mock
  BlobStoreManager manager;

  @Mock
  BlobStoreConfigurationStore store;

  @Mock
  BlobStoreQuotaService quotaService;

  @Mock(name = "azure cloud storage")
  ConnectionChecker connectionChecker;

  @Mock
  BlobStore noQuota;

  @Mock
  BlobStore passing;

  @Mock
  BlobStore failing;

  BlobStoreResource resource;

  @BeforeEach
  void setup() {
    lenient().when(quotaService.checkQuota(noQuota)).thenReturn(null);
    lenient().when(quotaService.checkQuota(passing)).thenReturn(new BlobStoreQuotaResult(false, "passing", "test"));
    lenient().when(quotaService.checkQuota(failing)).thenReturn(new BlobStoreQuotaResult(true, "failing", "test"));

    lenient().when(manager.get(ArgumentMatchers.eq("passing"))).thenReturn(passing);
    lenient().when(manager.get(ArgumentMatchers.eq("noQuota"))).thenReturn(noQuota);
    lenient().when(manager.get(ArgumentMatchers.eq("failing"))).thenReturn(failing);

    List<ConnectionChecker> connectionCheckers = new LinkedList<>();
    connectionCheckers.add(connectionChecker);

    resource = new BlobStoreResource(manager, store, quotaService, connectionCheckers);
  }

  @Test
  void passingTest() {
    BlobStoreQuotaResultXO resultXO = resource.quotaStatus("passing");
    assertFalse(resultXO.getIsViolation());
    assertEquals(resultXO.getBlobStoreName(), "passing");
  }

  @Test
  void failingTest() {
    BlobStoreQuotaResultXO resultXO = resource.quotaStatus("failing");
    assertTrue(resultXO.getIsViolation());
    assertEquals(resultXO.getBlobStoreName(), "failing");
  }

  @Test
  void noQuotaTest() {
    BlobStoreQuotaResultXO resultXO = resource.quotaStatus("noQuota");
    assertFalse(resultXO.getIsViolation());
    assertEquals(resultXO.getBlobStoreName(), "noQuota");
  }

  @Test
  void verifyConnectionTest() {
    when(connectionChecker.verifyConnection(ArgumentMatchers.any(String.class), ArgumentMatchers.any(Map.class)))
        .thenReturn(true);
    resource.verifyConnection(getBlobStoreConnectionXO());
  }

  @Test
  void verifyConnectionTestFail() {
    when(connectionChecker.verifyConnection(ArgumentMatchers.any(String.class), ArgumentMatchers.any(Map.class)))
        .thenThrow(new RuntimeException("Fake unsuccessful connection Exception"));
    BlobStoreConnectionXO blobStoreConnectionXO = getBlobStoreConnectionXO();
    assertThrows(WebApplicationException.class, () -> resource.verifyConnection(blobStoreConnectionXO));
  }

  @Test
  void verifyConnectionTestFailWithBlobStoreConnectionException() {
    when(connectionChecker.verifyConnection(ArgumentMatchers.any(String.class), ArgumentMatchers.any(Map.class)))
        .thenThrow(new BlobStoreConnectionException("Fake BlobStoreConnectionException"));
    WebApplicationException e =
        assertThrows(WebApplicationException.class, () -> resource.verifyConnection(getBlobStoreConnectionXO()));
    assertEquals(400, e.getResponse().getStatus());
    assertEquals("Fake BlobStoreConnectionException", e.getResponse().getEntity());
  }

  private static BlobStoreConnectionXO getBlobStoreConnectionXO() {
    Map<String, Object> connectionDetails = new HashMap<>();
    connectionDetails.put("accountName", "some account name");
    connectionDetails.put("accountKey", "some account key");
    connectionDetails.put("containerName", "some container name");
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    attributes.put("azure cloud storage", connectionDetails);
    BlobStoreConnectionXO blobStoreConnectionXO = new BlobStoreConnectionXO();
    blobStoreConnectionXO.setName("blobstoreName");
    blobStoreConnectionXO.setType("azure cloud storage");
    blobStoreConnectionXO.setAttributes(attributes);
    return blobStoreConnectionXO;
  }
}
