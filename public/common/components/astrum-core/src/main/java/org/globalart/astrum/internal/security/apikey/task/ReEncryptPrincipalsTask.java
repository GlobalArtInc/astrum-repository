
package org.globalart.astrum.internal.security.apikey.task;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.entity.Continuations;
import org.globalart.astrum.crypto.internal.PbeCipherFactory;
import org.globalart.astrum.crypto.internal.PbeCipherFactory.PbeCipher;
import org.globalart.astrum.crypto.secrets.EncryptedSecret;
import org.globalart.astrum.crypto.secrets.internal.EncryptionKeyList.FixedEncryption;
import org.globalart.astrum.crypto.secrets.internal.EncryptionKeyList.SecretEncryptionKey;
import org.globalart.astrum.crypto.secrets.internal.EncryptionKeySource;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.logging.task.ProgressLogIntervalHelper;
import org.globalart.astrum.logging.task.TaskLogType;
import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.TaskSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.bouncycastle.util.encoders.Hex;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.ASTRUM_SECURITY_SECRETS_ALGORITHM_NAMED_VALUE;
import static org.globalart.astrum.common.app.FeatureFlags.ASTRUM_SECURITY_SECRETS_ITERATIONS_NAMED_VALUE;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.IV;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.KEY_ITERATION_PHC;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.fromBase64;
import static org.globalart.astrum.crypto.internal.EncryptionHelper.toBase64;
import static org.globalart.astrum.crypto.internal.HashingHandlerFactoryImpl.DEFAULT_ITERATIONS_SHA1;
import static org.globalart.astrum.crypto.internal.HashingHandlerFactoryImpl.DEFAULT_ITERATIONS_SHA256;
import static org.globalart.astrum.crypto.internal.HashingHandlerFactoryImpl.KEY_FACTORY_ALGORITHM_SHA1;
import static org.globalart.astrum.crypto.internal.HashingHandlerFactoryImpl.KEY_FACTORY_ALGORITHM_SHA256;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@TaskLogging(TaskLogType.ASTRUM_LOG_ONLY)
public class ReEncryptPrincipalsTask
    extends TaskSupport
    implements Cancelable
{

  private static final int INTERVAL_IN_SECONDS = 60;

  private static final String SELECT = """
      SELECT principals, domain, username, access_key
      FROM api_key_v2
      ORDER BY username, principals, domain
      LIMIT ? OFFSET ?
      """;

  private static final String UPDATE = """
      UPDATE api_key_v2
      SET principals = ?
      WHERE domain = ? AND username = ? AND access_key = ?
      """;

  private final DataSessionSupplier sessionSupplier;

  private final EncryptionKeySource encryptionKeySource;

  private final PbeCipherFactory pbeCipherFactory;

  private final String password;

  private final String salt;

  private final String iv;

  private final String astrumSecretsAlgorithm;

  private final Integer astrumSecretsIterations;

  @Autowired
  public ReEncryptPrincipalsTask(
      final DataSessionSupplier sessionSupplier,
      final EncryptionKeySource encryptionKeySource,
      final PbeCipherFactory pbeCipherFactory,
      @Value("${astrum.mybatis.cipher.password:changeme}") final String password,
      @Value("${astrum.mybatis.cipher.salt:changeme}") final String salt,
      @Value("${astrum.mybatis.cipher.iv:0123456789ABCDEF}") final String iv,
      @Value(ASTRUM_SECURITY_SECRETS_ALGORITHM_NAMED_VALUE) final String astrumSecretsAlgorithm,
      @Value(ASTRUM_SECURITY_SECRETS_ITERATIONS_NAMED_VALUE) final Integer astrumSecretsIterations)
  {
    this.sessionSupplier = checkNotNull(sessionSupplier);
    this.encryptionKeySource = checkNotNull(encryptionKeySource);
    this.pbeCipherFactory = checkNotNull(pbeCipherFactory);
    this.password = password;
    this.salt = salt;
    this.iv = iv;
    this.astrumSecretsAlgorithm = astrumSecretsAlgorithm;
    this.astrumSecretsIterations = astrumSecretsIterations;
  }

  @Override
  public String getMessage() {
    return "Re-encrypting principals";
  }

  @Override
  protected Object execute() throws Exception {
    long start = System.currentTimeMillis();
    log.info("Started re-encryption principals");
    SecretEncryptionKey keyForDecryption = new SecretEncryptionKey(null, password);
    SecretEncryptionKey keyForEncryption = new SecretEncryptionKey(null, password);

    String saltForDecryption = salt;
    String saltForEncryption = salt;
    String ivForDecryption = iv;
    String ivForEncryption = iv;

    String algorithmForDecryption = getValueFromTaskConfiguration("algorithmForDecryption", astrumSecretsAlgorithm);
    Integer iterationsForDecryption = Integer.parseInt(getValueFromTaskConfiguration("iterationsForDecryption",
        defaultIterationByAlgorithm(algorithmForDecryption).toString()));
    Integer iterationsForEncryption = astrumSecretsIterations != null
        ? astrumSecretsIterations
        : defaultIterationByAlgorithm(astrumSecretsAlgorithm);

    Optional<FixedEncryption> previousEncryptionConfigOptional = encryptionKeySource.getPreviousFixedEncryption();
    if (previousEncryptionConfigOptional.isPresent()) {
      FixedEncryption config = previousEncryptionConfigOptional.get();
      keyForDecryption =
          config.getKeyId() != null ? encryptionKeySource.getKey(config.getKeyId()).orElseThrow() : keyForDecryption;
      saltForDecryption = config.getSalt() != null ? config.getSalt() : saltForDecryption;
      ivForDecryption = config.getIv() != null ? config.getIv() : ivForDecryption;
    }

    Optional<FixedEncryption> encryptionConfigOptional = encryptionKeySource.getFixedEncryption();
    if (encryptionConfigOptional.isPresent()) {
      FixedEncryption config = encryptionConfigOptional.get();
      keyForEncryption =
          config.getKeyId() != null ? encryptionKeySource.getKey(config.getKeyId()).orElseThrow() : keyForEncryption;
      saltForEncryption = config.getSalt() != null ? config.getSalt() : saltForEncryption;
      ivForEncryption = config.getIv() != null ? config.getIv() : ivForEncryption;
    }

    PbeCipher cipherForEncryption =
        pbeCipherFactory.create(keyForEncryption, saltForEncryption, ivForEncryption, iterationsForEncryption);
    try (ProgressLogIntervalHelper progressLogger = new ProgressLogIntervalHelper(log, INTERVAL_IN_SECONDS);
        Connection conn = sessionSupplier.openConnection(DEFAULT_DATASTORE_NAME);
        PreparedStatement select = conn.prepareStatement(SELECT);
        PreparedStatement update = conn.prepareStatement(UPDATE)) {
      int offset = 0;
      int processedCount = 0;
      int failedCount = 0;
      while (true) {
        select.setInt(1, Continuations.BROWSE_LIMIT);
        select.setInt(2, offset);
        try (ResultSet results = select.executeQuery()) {
          if (!results.isBeforeFirst()) {
            break;
          }

          while (results.next()) {
            String domain = results.getString("domain");
            String username = results.getString("username");
            String accessKey = results.getString("access_key");
            byte[] principals = results.getBytes("principals");
            try {
              byte[] decrypted =
                  decrypt(keyForDecryption, algorithmForDecryption, saltForDecryption, ivForDecryption,
                      iterationsForDecryption, principals);
              byte[] encrypted = reEncrypt(cipherForEncryption, decrypted);
              updatePrincipal(update, domain, username, accessKey, encrypted);
              processedCount++;
            }
            catch (Exception e) {
              failedCount++;
              log.error("An error occurred trying to re-encrypt principals of username: {}.", username, e);
            }
          }

          progressLogger.info("Re-encrypted principals {}, Failed principals {}.", processedCount, failedCount);
          offset += Continuations.BROWSE_LIMIT;
        }
      }

      long finish = System.currentTimeMillis();
      long timeElapsed = finish - start;
      log.info(
          "The re-encryption principals has been completed. Re-encrypted principals {}, Failed principals {}. It took {} milliseconds.",
          processedCount, failedCount, timeElapsed);
      return processedCount;
    }
  }

  private byte[] decrypt(
      final SecretEncryptionKey secretKeyToDecrypt,
      final String algorithm,
      final String salt,
      final String iv,
      final Integer keyIterations,
      final byte[] principalsBytes)
  {
    Map<String, String> attributes = new HashMap<>(Map.of(
        IV, Hex.toHexString(iv.getBytes())));
    if (keyIterations != null) {
      attributes.put(KEY_ITERATION_PHC, String.valueOf(keyIterations));
    }

    EncryptedSecret encryptedSecret = new EncryptedSecret(algorithm, null,
        toBase64(salt.getBytes()),
        toBase64(principalsBytes),
        attributes);
    PbeCipher cipherToDecrypt = pbeCipherFactory.create(secretKeyToDecrypt, encryptedSecret.toPhcString());
    return cipherToDecrypt.decrypt();
  }

  private static byte[] reEncrypt(
      final PbeCipher cipherToEncrypt,
      final byte[] decrypted)
  {
    EncryptedSecret encryptedSecret = cipherToEncrypt.encrypt(decrypted);
    String base64Value = encryptedSecret.getValue();
    return fromBase64(base64Value);
  }

  private static void updatePrincipal(
      final PreparedStatement update,
      final String domain,
      final String username,
      final String accessKey,
      final byte[] encrypted) throws SQLException
  {
    update.setBytes(1, encrypted);
    update.setString(2, domain);
    update.setString(3, username);
    update.setString(4, accessKey);
    update.executeUpdate();
  }

  private static Integer defaultIterationByAlgorithm(final String algorithm) {
    return switch (algorithm) {
      case KEY_FACTORY_ALGORITHM_SHA1 -> DEFAULT_ITERATIONS_SHA1;
      case KEY_FACTORY_ALGORITHM_SHA256 -> DEFAULT_ITERATIONS_SHA256;
      default -> throw new IllegalArgumentException(
          String.format("Unsupported algorithm: '%s'. Supported algorithms are: '%s', '%s'",
              algorithm, KEY_FACTORY_ALGORITHM_SHA1, KEY_FACTORY_ALGORITHM_SHA256));
    };
  }

  private String getValueFromTaskConfiguration(final String configurationKey, final String defaultValue) {
    String value = taskConfiguration().getString(configurationKey);
    if (value == null || value.isBlank()) {
      value = defaultValue;
    }

    return value;
  }
}
