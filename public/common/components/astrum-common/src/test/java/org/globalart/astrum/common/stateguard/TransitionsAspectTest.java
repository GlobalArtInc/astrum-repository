
package org.globalart.astrum.common.stateguard;

import java.lang.reflect.Method;

import org.aspectj.lang.ProceedingJoinPoint;
import org.aspectj.lang.reflect.MethodSignature;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.NEW;
import static org.globalart.astrum.common.stateguard.StateGuardTest.State.INITIALISED;

@RunWith(MockitoJUnitRunner.Silent.class)
public class TransitionsAspectTest

{
  private TransitionsAspect transitionsAspect;

  @Mock
  private ProceedingJoinPoint joinPoint;

  @Mock
  private MethodSignature methodSignature;

  @Mock
  private StateGuardAware target;

  @Mock
  private StateGuard stateGuard;

  @Mock
  private Transition transition;

  @Mock
  private Transition transition1;

  @Before
  public void setUp() {
    transitionsAspect = new TransitionsAspect();
  }

  @Test
  public void testAroundTransitionsMethod() throws Throwable {
    // Mock method and annotation
    Method method = TestClass.class.getMethod("testMethod");
    Transitions transitions = method.getAnnotation(Transitions.class);

    // Mock joinPoint behavior
    when(transition.from(any())).thenReturn(transition1);

    when(joinPoint.getTarget()).thenReturn(target);
    when(joinPoint.getSignature()).thenReturn(methodSignature);
    when(methodSignature.getMethod()).thenReturn(method);
    when(target.getStateGuard()).thenReturn(stateGuard);
    when(stateGuard.transition(transitions.to(), transitions.silent(), transitions.ignore(),
        transitions.requiresWriteLock()))
            .thenReturn(transition);

    // Mock transition behavior
    when(transition1.run(any())).thenAnswer(invocation -> {
      Action<?> action = invocation.getArgument(0);
      return action.run();
    });

    // Execute the aspect
    transitionsAspect.aroundTransitionsMethod(joinPoint, transitions);

    // Verify interactions
    verify(stateGuard).transition(transitions.to(), transitions.silent(), transitions.ignore(),
        transitions.requiresWriteLock());
    verify(transition1).run(any());
    verify(joinPoint).proceed();
  }

  // Test class with a method annotated with @Transitions
  static class TestClass
  {
    @Transitions(to = INITIALISED, from = NEW)
    public void testMethod() {
    }
  }
}
