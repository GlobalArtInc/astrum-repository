
package org.globalart.astrum.self.hosted.node;

import java.security.cert.Certificate;
import java.security.cert.CertificateEncodingException;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.internal.node.NodeIdEncoding;
import org.globalart.astrum.internal.node.NodeIdInitializer;
import org.globalart.astrum.node.datastore.NodeIdStore;
import org.globalart.astrum.ssl.KeyStoreManager;
import org.globalart.astrum.ssl.KeystoreException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Migrates the legacy on disk node identifier to the database where it may be used to identify a single node, or a
 * cluster.
 */
@Component
public class NodeIdInitializerImpl
    implements NodeIdInitializer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String NODE_KEY_STORE_TYPE = "node";

  private final NodeIdStore nodeIdStore;

  private final Provider<KeyStoreManager> keyStoreProvider;

  @Autowired
  public NodeIdInitializerImpl(
      @Qualifier(NODE_KEY_STORE_TYPE) final Provider<KeyStoreManager> keyStoreProvider,
      final NodeIdStore nodeIdStore)
  {
    this.nodeIdStore = checkNotNull(nodeIdStore);
    this.keyStoreProvider = checkNotNull(keyStoreProvider);
  }

  public void initialize() {
    if (!nodeIdStore.get().isPresent()) {
      migrateNodeId();
    }
  }

  private void migrateNodeId() {
    log.info("No node-id found. Attempting to migrate from KeyStore");
    KeyStoreManager keyStoreManager = keyStoreProvider.get();

    if (keyStoreManager == null || !keyStoreManager.isKeyPairInitialized()) {
      generateNodeId();
      return;
    }

    // Migrating an existing key
    log.info("Migrating node-id to database");

    try {
      Certificate certificate = keyStoreManager.getCertificate();
      log.trace("Certificate:\n{}", certificate);

      String id = NodeIdEncoding.nodeIdForCertificate(certificate);

      nodeIdStore.set(id);
    }
    catch (KeystoreException | CertificateEncodingException e) {
      generateNodeId();
    }
  }

  private void generateNodeId() {
    log.info("Unable to get node-id from KeyStore - Generating a new node-id");
    nodeIdStore.getOrCreate();
  }
}
