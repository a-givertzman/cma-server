# Список тестов

Это перечень тестов которые необходимы для надежной работы вычислительного графа

## Graph correctness

1. Cycle detection

Проверяет обнаружение циклических зависимостей.

2. Self-reference graph

Проверяет узел, зависящий сам от себя.

3. Deep recursion graph

Проверяет stack overflow и глубину eval.

4. Multi-root graph

Проверяет корректность нескольких независимых корней.

5. Wildcard every propagation

Проверяет корректность every.

6. Type-filtered every

Проверяет фильтрацию point int every.

7. Deterministic ordering

Проверяет одинаковость результата независимо от порядка событий.

## Concurrency

8. Event storm test

Миллионы событий быстрее consumer. Проверка memory growth.

9. Slow function test

Одна тяжелая функция блокирует pipeline.

10. Shutdown during load

Остановка во время активной обработки.

11. Concurrent subscriptions

Параллельные подписки/отписки.

12. Queue overflow

Поведение при переполнении.

13. Backpressure test

Проверка деградации при перегрузке.

14. Runtime borrow panic

Проверка RefCell borrow conflicts.

---

## Fault tolerance

15. Invalid config

Некорректный YAML/config.

16. Missing input

Отсутствующий dependency.

17. Invalid type conversion

Неверные типы.

18. Function panic isolation

Panic внутри функции не должен валить runtime.

19. Broken subscription source

Источник перестал отвечать.

20. Queue disconnect

Разрыв channel.

---

## Performance

21. Large graph benchmark

Тысячи узлов.

22. High fan-out graph

Один input -> тысячи downstream.

23. Recalculation efficiency

Проверка memoization.

24. Allocation pressure

Измерение allocations/clones.

25. Logging overhead

Нагрузка при debug/trace.

---

## Stateful functions

26. Timer correctness

Корректность таймеров.

27. Edge detection correctness

Правильность rising/falling edge.

28. Reset semantics

Сброс состояния.

29. Cache invalidation

Инвалидаци