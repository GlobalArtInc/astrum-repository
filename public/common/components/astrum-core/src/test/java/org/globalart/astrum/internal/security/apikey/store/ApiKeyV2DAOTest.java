
package org.globalart.astrum.internal.security.apikey.store;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.Arrays;
import java.util.Collection;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.stream.Collectors;

import org.globalart.astrum.crypto.LegacyCipherFactory.PbeCipher;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsFactory;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.datastore.mybatis.handlers.PrincipalCollectionTypeHandler;
import org.globalart.astrum.datastore.mybatis.handlers.SecretTypeHandler;
import org.globalart.astrum.internal.security.apikey.ApiKeyInternal;
import org.globalart.astrum.testdb.DataSessionRule;

import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.assertj.db.type.Table;
import org.hamcrest.Description;
import org.hamcrest.Matcher;
import org.hamcrest.TypeSafeMatcher;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.mockito.Mock;
import org.mockito.internal.stubbing.defaultanswers.ReturnsMocks;

import static org.assertj.db.api.Assertions.assertThat;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.internal.security.apikey.store.ApiKeyStoreV2Impl.accessKey;
import static org.globalart.astrum.internal.security.apikey.store.ApiKeyStoreV2Impl.secret;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ApiKeyV2DAOTest
{
  private static final String DOMAIN = "a domain";

  private static final String ANOTHER_DOMAIN = "another domain";

  private static final char[] API_KEY1 = "api_key;:\"|\\}{}[]+-=-=3/><+\"|:@!^%$£&*~`_+_o".toCharArray();

  private static final char[] API_KEY2 = "api_key2;:\"|\\}{}[]+-=-=3/><+\"|:@!^%$£&*~`_+_o".toCharArray();

  private static final char[] API_KEY3 = "api_key3;:\"|\\}{}[]+-=-=3/><+\"|:@!^%$£&*~`_+_o".toCharArray();

  private static final char[] API_KEY4 = "api_key4;:\"|\\}{}[]+-=-=3/><+\"|:@!^%$£&*~`_+_o".toCharArray();

  private static final String A_PRINCIPAL = "principal1";

  private static final String ANOTHER_PRINCIPAL = "another_principal1";

  private static final String A_REALM = "realm1";

  private static final String ANOTHER_REALM = "another_realm";

  private Map<String, Secret> storedSecrets = new HashMap<>();

  private SecretsFactory factory = mock(SecretsFactory.class, new ReturnsMocks());

  private final PrincipalCollectionTypeHandler principalHandler = new PrincipalCollectionTypeHandler();

  @Mock
  private PbeCipher cipher;

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule()
      .access(ApiKeyV2DAO.class)
      .handle(new SecretTypeHandler(factory))
      .handle(principalHandler);

  @Before
  public void setup() {
    when(factory.from(anyString()))
        .thenAnswer(i -> storedSecrets.get(i.getArguments()[0]));

    when(cipher.encrypt(any())).thenAnswer(i -> i.getArguments()[0]);
    when(cipher.decrypt(any())).thenAnswer(i -> i.getArguments()[0]);
  }

  /*
   * create should successfully create and fetch an ApiKey record
   */
  @Test
  public void testCreate() {
    ApiKeyV2Data apiKeyEntity = anApiKeyEntity(API_KEY1, DOMAIN, A_PRINCIPAL);

    withDao(dao -> dao.save(apiKeyEntity));

    ApiKeyInternal savedApiKey = findApiKey(DOMAIN, A_REALM, A_PRINCIPAL).get();
    assertSavedApiKey(savedApiKey, API_KEY1);
  }

  /*
   * Create the same user in different realms
   */
  @Test
  public void testCreate_differentRealm() {
    OffsetDateTime created = OffsetDateTime.now();
    ApiKeyV2Data apiKeyEntity = anApiKeyEntity(API_KEY1, DOMAIN, A_PRINCIPAL, A_REALM, created);
    ApiKeyV2Data apiKeyEntity2 = anApiKeyEntity(API_KEY2, DOMAIN, A_PRINCIPAL, ANOTHER_REALM, created);

    withDao(dao -> dao.save(apiKeyEntity));
    withDao(dao -> dao.save(apiKeyEntity2));

    assertThat(table()).hasNumberOfRows(2);
  }

  /*
   * update should successfully update matching ApiKey record
   */
  @Test
  public void testUpdatePrincipal() {
    ApiKeyV2Data apiKeyEntity = anApiKeyEntity(API_KEY1, DOMAIN, A_PRINCIPAL);
    ApiKeyV2Data anotherApiKeyEntity = anApiKeyEntity(API_KEY2, DOMAIN, ANOTHER_PRINCIPAL);
    withDao(dao -> dao.save(apiKeyEntity));
    withDao(dao -> dao.save(anotherApiKeyEntity));

    ApiKeyInternal savedApiKey1 = findApiKey(DOMAIN, A_REALM, A_PRINCIPAL).get();
    ApiKeyInternal savedApiKey2 = findApiKey(DOMAIN, A_REALM, ANOTHER_PRINCIPAL).get();

    // retrieve
    assertSavedApiKey(savedApiKey1, API_KEY1);
    assertSavedApiKey(savedApiKey2, API_KEY2);

    // make a change
    savedApiKey1.setPrincipals(principalCollection(apiKeyEntity.getUsername(), ANOTHER_REALM));
    withDao(dao -> dao.updatePrincipal((ApiKeyV2Data) savedApiKey1));

    ApiKeyInternal updatedApiKey = findApiKey(DOMAIN, ANOTHER_REALM, A_PRINCIPAL).get();
    assertNotNull(updatedApiKey);
  }

  /*
   * delete should successfully delete matching ApiKey record
   */
  @Test
  public void testDelete() {
    ApiKeyV2Data entity1 = anApiKeyEntity(API_KEY1, DOMAIN, A_PRINCIPAL);
    ApiKeyV2Data entity2 = anApiKeyEntity(API_KEY2, DOMAIN, ANOTHER_PRINCIPAL);
    withDao(dao -> dao.save(entity1));
    withDao(dao -> dao.save(entity2));

    Optional<ApiKeyV2Data> savedApiKey1 = findApiKey(DOMAIN, A_REALM, A_PRINCIPAL);
    Optional<ApiKeyV2Data> savedApiKey2 = findApiKey(DOMAIN, A_REALM, ANOTHER_PRINCIPAL);

    assertTrue(savedApiKey1.isPresent());
    assertTrue(savedApiKey2.isPresent());

    withDao(dao -> dao.deleteApiKey(entity1));
    savedApiKey1 = findApiKey(DOMAIN, A_REALM, A_PRINCIPAL);
    savedApiKey2 = findApiKey(DOMAIN, A_REALM, ANOTHER_PRINCIPAL);
    assertFalse(savedApiKey1.isPresent());
    assertTrue(savedApiKey2.isPresent());
  }

  /*
   * browse should fetch all records
   */
  @Test
  public void testBrowsePrincipals() {
    ApiKeyV2Data entity1 = anApiKeyEntity(API_KEY1, DOMAIN, A_PRINCIPAL);
    ApiKeyV2Data entity2 = anApiKeyEntity(API_KEY2, DOMAIN, ANOTHER_PRINCIPAL);
    withDao(dao -> dao.save(entity1));
    withDao(dao -> dao.save(entity2));

    Collection<PrincipalCollection> allRecords =
        (Collection<PrincipalCollection>) callDao(dao -> dao.browsePrincipals());
    assertThat(allRecords, hasSize(2));
    assertThat(allRecords.stream().map(PrincipalCollection::getPrimaryPrincipal).collect(Collectors.toList()),
        containsInAnyOrder(A_PRINCIPAL, ANOTHER_PRINCIPAL));
  }

  /*
   * findApiKey should fetch records matching given domain and primary principal
   */
  @Test
  public void testFindApiKey() {
    withDao(dao -> dao.save(anApiKeyEntity(API_KEY1, DOMAIN, A_PRINCIPAL)));
    withDao(dao -> dao.save(anApiKeyEntity(API_KEY2, DOMAIN, ANOTHER_PRINCIPAL)));
    withDao(dao -> dao.save(anApiKeyEntity(API_KEY3, ANOTHER_DOMAIN, A_PRINCIPAL)));
    withDao(dao -> dao.save(anApiKeyEntity(API_KEY4, ANOTHER_DOMAIN, ANOTHER_PRINCIPAL)));

    Optional<ApiKeyV2Data> result = callDao(dao -> dao.findApiKey(DOMAIN, A_PRINCIPAL)).stream().findFirst();
    assertSavedApiKey(result.get(), API_KEY1);

    //
    result = callDao(dao -> dao.findApiKey(DOMAIN, ANOTHER_PRINCIPAL)).stream().findFirst();
    assertSavedApiKey(result.get(), API_KEY2);

    result = callDao(dao -> dao.findApiKey(ANOTHER_DOMAIN, A_PRINCIPAL)).stream().findFirst();
    assertSavedApiKey(result.get(), API_KEY3);
  }

  /*
   * findPrincipals should fetch records matching given domain and api key
   */
  @Test
  public void testFindPrincipals() {
    ApiKeyV2Data apiKeyEntity = anApiKeyEntity(API_KEY1, DOMAIN, A_PRINCIPAL);
    ApiKeyV2Data anotherApiKeyEntity = anApiKeyEntity(API_KEY2, DOMAIN, ANOTHER_PRINCIPAL);
    withDao(dao -> dao.save(apiKeyEntity));
    withDao(dao -> dao.save(anotherApiKeyEntity));

    Optional<ApiKeyInternal> result = callDao(dao -> dao.findPrincipals(DOMAIN, accessKey(API_KEY1)));
    assertThat(result.get().getPrincipals().getPrimaryPrincipal(), is(A_PRINCIPAL));

    result = callDao(dao -> dao.findPrincipals(DOMAIN, accessKey(API_KEY2)));
    assertThat(result.get().getPrincipals().getPrimaryPrincipal(), is(ANOTHER_PRINCIPAL));
  }

  @Test
  public void testCount() {
    ApiKeyV2Data entity1 = anApiKeyEntity(API_KEY1, DOMAIN, A_PRINCIPAL);
    ApiKeyV2Data entity2 = anApiKeyEntity(API_KEY2, ANOTHER_DOMAIN, ANOTHER_PRINCIPAL);
    withDao(dao -> dao.save(entity1));
    withDao(dao -> dao.save(entity2));

    assertThat(callDao(dao -> dao.count(DOMAIN)), is(1));
  }

  @Test
  public void testBrowse() {
    ApiKeyV2Data entity1 = anApiKeyEntity(API_KEY1, DOMAIN, A_PRINCIPAL);
    ApiKeyV2Data entity2 = anApiKeyEntity(API_KEY2, DOMAIN, ANOTHER_PRINCIPAL);
    withDao(dao -> dao.save(entity1));
    withDao(dao -> dao.save(entity2));
    withDao(dao -> dao.save(anApiKeyEntity(API_KEY2, ANOTHER_DOMAIN, A_PRINCIPAL)));

    assertThat(callDao(dao -> dao.browse(DOMAIN)), containsInAnyOrder(token(entity1), token(entity2)));
  }

  @Test
  public void testBrowseCreatedAfter() {
    ApiKeyV2Data entity1 = anApiKeyEntity(API_KEY1, DOMAIN, A_PRINCIPAL);
    ApiKeyV2Data entity2 = anApiKeyEntity(API_KEY2, DOMAIN, ANOTHER_PRINCIPAL);
    ApiKeyV2Data entity3 = anApiKeyEntity(API_KEY2, ANOTHER_DOMAIN, A_PRINCIPAL);

    OffsetDateTime entity1Date = OffsetDateTime.of(2021, 1, 1, 0, 0, 0, 0, ZoneOffset.UTC);
    OffsetDateTime entity2Date = OffsetDateTime.of(2022, 1, 1, 0, 0, 0, 0, ZoneOffset.UTC);

    entity1.setCreated(entity1Date);
    entity2.setCreated(entity2Date);

    withDao(dao -> dao.save(entity1));
    withDao(dao -> dao.save(entity2));
    withDao(dao -> dao.save(anApiKeyEntity(API_KEY2, ANOTHER_DOMAIN, A_PRINCIPAL)));

    assertThat(callDao(dao -> dao.browseCreatedAfter(DOMAIN, entity1Date)), contains(token(entity2), token(entity3)));
  }

  @Test
  public void testBrowseCreatedBefore() {
    ApiKeyV2Data entity1 = anApiKeyEntity(API_KEY1, DOMAIN, A_PRINCIPAL);
    ApiKeyV2Data entity2 = anApiKeyEntity(API_KEY2, DOMAIN, ANOTHER_PRINCIPAL);

    OffsetDateTime entity1Date = OffsetDateTime.of(2021, 1, 1, 0, 0, 0, 0, ZoneOffset.UTC);
    OffsetDateTime entity2Date = OffsetDateTime.of(2022, 1, 1, 0, 0, 0, 0, ZoneOffset.UTC);

    entity1.setCreated(entity1Date);
    entity2.setCreated(entity2Date);

    withDao(dao -> dao.save(entity1));
    withDao(dao -> dao.save(entity2));
    withDao(dao -> dao.save(anApiKeyEntity(API_KEY2, ANOTHER_DOMAIN, A_PRINCIPAL)));

    assertThat(callDao(dao -> dao.browseByDomainCreatedBefore(DOMAIN, entity2Date.minusSeconds(1L))),
        contains(token(entity1)));
  }

  private ApiKeyV2Data anApiKeyEntity(
      final char[] apiKey,
      final String domain,
      final String primaryPrincipal)
  {
    OffsetDateTime created = OffsetDateTime.now();
    return anApiKeyEntity(apiKey, domain, primaryPrincipal, A_REALM, created);
  }

  private ApiKeyV2Data anApiKeyEntity(
      final char[] apiKey,
      final String domain,
      final String primaryPrincipal,
      final String realm,
      final OffsetDateTime created)
  {
    PrincipalCollection principalCollection = principalCollection(primaryPrincipal, realm);
    return anApiKeyEntity(apiKey, domain, principalCollection, created);
  }

  private ApiKeyV2Data anApiKeyEntity(
      final char[] apiKey,
      final String domain,
      final PrincipalCollection principal,
      final OffsetDateTime created)
  {
    ApiKeyV2Data data = new ApiKeyV2Data();
    data.setDomain(domain);
    data.setPrincipals(principal);
    data.setAccessKey(accessKey(apiKey));

    setSecret(data, apiKey);
    data.setCreated(created);
    return data;
  }

  private void setSecret(final ApiKeyV2Data data, final char[] apiKey) {
    Secret secret = mock(Secret.class);
    String key = "_" + storedSecrets.size();
    char[] dataChars = secret(apiKey);
    storedSecrets.put(key, secret);

    when(secret.decrypt()).thenReturn(dataChars);

    when(secret.getId()).thenReturn(key);
    data.setSecret(secret);
  }

  private Optional<ApiKeyV2Data> findApiKey(final String domain, final String realm, final String primaryPrincipal) {
    return callDao(dao -> dao.findApiKey(domain, primaryPrincipal)).stream()
        .filter(key -> key.getPrincipals().getRealmNames().contains(realm))
        .findFirst();
  }

  private Table table() {
    DataStore<?> dataStore = sessionRule.getDataStore(DEFAULT_DATASTORE_NAME).orElseThrow(RuntimeException::new);
    return new Table(dataStore.getDataSource(), "api_key_v2");
  }

  private <E> E callDao(final Function<ApiKeyV2DAO, E> consumer) {
    try (DataSession<?> session = sessionRule.openSerializableTransactionSession(DEFAULT_DATASTORE_NAME)) {
      E result = consumer.apply(session.access(ApiKeyV2DAO.class));
      session.getTransaction().commit();
      return result;
    }
  }

  private void withDao(final Consumer<ApiKeyV2DAO> consumer) {
    try (DataSession<?> session = sessionRule.openSerializableTransactionSession(DEFAULT_DATASTORE_NAME)) {
      consumer.accept(session.access(ApiKeyV2DAO.class));
      session.getTransaction().commit();
    }
  }

  private static SimplePrincipalCollection principalCollection(final String principal, final String realm) {
    return new SimplePrincipalCollection(principal, realm);
  }

  private static void assertSavedApiKey(final ApiKeyInternal actualApiKey, final char[] expectedApiKey) {
    assertThat(new String(actualApiKey.getApiKey()), is(new String(expectedApiKey)));
  }

  private static Matcher<ApiKeyInternal> token(final ApiKeyInternal key) {
    return new TypeSafeMatcher<ApiKeyInternal>(ApiKeyInternal.class)
    {
      @Override
      public void describeTo(final Description description) {
        description.appendValue(key);
      }

      @Override
      protected boolean matchesSafely(final ApiKeyInternal item) {
        return item.getPrincipals().equals(key.getPrincipals()) && item.getDomain().equals(key.getDomain())
            && Arrays.equals(item.getApiKey(), key.getApiKey());
      }
    };
  }
}
