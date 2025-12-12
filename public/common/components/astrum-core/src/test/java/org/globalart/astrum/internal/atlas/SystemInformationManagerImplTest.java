
package org.globalart.astrum.internal.atlas;

import java.util.Map;

import javax.cache.Cache;
import javax.cache.configuration.MutableConfiguration;

import org.globalart.astrum.cache.CacheHelper;
import org.globalart.astrum.common.atlas.SystemInformationGenerator;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;
import com.google.common.collect.ImmutableMap;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.argThat;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.mockito.internal.verification.VerificationModeFactory.times;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SystemInformationManagerImplTest
{
  private static final Map<String, Object> SYSTEM_INFO_REPORT =
      ImmutableMap.of("systemInfo", ImmutableMap.of("nodeId", "testId"));

  private static final String CACHE_NAME = "SYSTEM_INFORMATION";

  private static final String SYSTEM_INFO_KEY = "systemInfo";

  @Mock
  private SystemInformationGenerator systemInformationGenerator;

  @Mock
  private CacheHelper cacheHelper;

  @Mock
  private Cache<String, Map<String, Object>> cache;

  private SystemInformationManagerImpl underTest;

  @Before
  public void setup() {
    underTest = new SystemInformationManagerImpl(systemInformationGenerator, cacheHelper, 300);

    when(systemInformationGenerator.report()).thenReturn(SYSTEM_INFO_REPORT);
    when(cacheHelper.maybeCreateCache(any(String.class), any(MutableConfiguration.class))).thenReturn(cache);

    underTest.doStart();
  }

  @After
  public void clear() {
    underTest.doStop();
  }

  @Test
  public void testGenerateAndCacheSystemInfoCorrectly() {
    when(cache.get(SYSTEM_INFO_KEY)).thenReturn(null);

    verify(cacheHelper, times(1)).maybeCreateCache(argThat(arg -> arg.equals(CACHE_NAME)),
        any(MutableConfiguration.class));

    Map<String, Object> systemInfo = underTest.getSystemInfo();
    verify(cache, times(1)).get(SYSTEM_INFO_KEY);
    verify(systemInformationGenerator, times(1)).report();
    verify(cache, times(1)).put(SYSTEM_INFO_KEY, SYSTEM_INFO_REPORT);
    assertThat(systemInfo, notNullValue());
    assertThat(systemInfo, is(SYSTEM_INFO_REPORT));
  }

  @Test
  public void testGetCachedSystemInfo() {
    when(cache.get(SYSTEM_INFO_KEY)).thenReturn(SYSTEM_INFO_REPORT);

    verify(cacheHelper, times(1)).maybeCreateCache(argThat(arg -> arg.equals(CACHE_NAME)),
        any(MutableConfiguration.class));

    Map<String, Object> systemInfo = underTest.getSystemInfo();
    verify(cache, times(1)).get(SYSTEM_INFO_KEY);
    verify(systemInformationGenerator, never()).report();
    verify(cache, never()).put(SYSTEM_INFO_KEY, SYSTEM_INFO_REPORT);
    assertThat(systemInfo, notNullValue());
    assertThat(systemInfo, is(SYSTEM_INFO_REPORT));
  }
}
