#[cfg(test)]

use std::{sync::Once, thread, time::Duration};
use debugging::session::{DebugSession, LogLevel};
use sal_core::dbg::Dbg;

///
///
static INIT: Once = Once::new();
///
/// once called initialisation
fn init_once() {
    INIT.call_once(|| {
        // implement your initialisation code to be called only once for current test file
    })
}
///
/// returns:
///  - ...
fn init_each() {}
///
/// If thread is already finished, join() or wait() don't returns error
#[ignore = "Learn - all must be ignored"]
#[test]
fn exiting() {
    DebugSession::new().filter(LogLevel::Debug).init().unwrap();
    init_once();
    init_each();
    let dbg = Dbg::own("kanal_channel_test");
    log::debug!("{}", dbg);
    let (send, recv) = kanal::unbounded();
    let handler = thread::spawn(move|| {
        log::info!("thread | Started");
        _ = send.send(0);
        for i in 0..10 {
            log::info!("thread | iteration: {}", i);
        }
        std::thread::sleep(Duration::from_millis(100));
        drop(send);
        log::info!("thread | Finished");
    });
    loop {
        log::info!("{dbg} | loop | receiving...");
        match recv.recv() {
            Ok(v) => {
                log::warn!("{dbg} | Recv value: '{:?}'", v);
            }
            Err(err) => {
                log::warn!("{dbg} | Recv error: {:?}", err);
                break;
            }
        }
    }
    log::info!("{dbg} | loop | exited");
    std::thread::sleep(Duration::from_millis(100));
    handler.join().unwrap();
    // assert!(result == target, "\nresult: {:?}\ntarget: {:?}", result, target);
}
///
/// ### Тестирую итератор `try_recv` на ресивере
#[ignore = "Learn - all must be ignored"]
#[test]
fn try_recv_test() {
    // Создаем канал емкостью 10 элементов
    let (tx, rx) = crate::domain::bounded::<(u32, f64)>(10);

    // 1. Кладим в канал 3 элемента
    tx.send((1, 10.5)).unwrap();
    tx.send((2, 5.0)).unwrap();
    tx.send((1, 4.5)).unwrap();

    println!("Данные отправлены. Вызываем rx.try_recv()...");

    // Запускаем в отдельном потоке, чтобы убедиться, что он НЕ зависнет
    let handle = std::thread::spawn(move || {
        // ВНИМАНИЕ: вызываем try_fold напрямую на КАНАЛЕ (rx), а не на итераторе!
        // Родной метод try_fold в kanal возвращает Result<Acc, ReceiveError>
        let mut ids = Vec::with_capacity(rx.len());
        let mut deprecations = Vec::with_capacity(rx.len());
        while let Ok(Some((id, dep))) = rx.try_recv() {
            ids.push(id);
            deprecations.push(dep);
        }
        (ids, deprecations)
    });
    // Даем потоку немного времени на выполнение
    std::thread::sleep(Duration::from_millis(200));

    if handle.is_finished() {
        // Поток успешно завершился сам, так как try_fold не блокирует поток
        let fold_res = handle.join().unwrap();
        println!("✅ Успех! try_recv() мгновенно вышел. Результат: {:?}", fold_res);
    } else {
        println!("try_recv КЛИНИТ!");
    }
}
///
/// ### Тестирую итератор `drain_into` на ресивере
#[ignore = "Learn - all must be ignored"]
#[test]
fn drain_into_test() {
    // Создаем канал емкостью 10 элементов
    let (tx, rx) = crate::domain::bounded(10);

    // 1. Кладим в канал 3 элемента
    tx.send((1, 10.5)).unwrap();
    tx.send((2, 5.0)).unwrap();
    tx.send((1, 4.5)).unwrap();

    println!("Данные отправлены. Вызываем rx.drain_into()...");

    // Запускаем в отдельном потоке, чтобы убедиться, что он НЕ зависнет
    let handle = std::thread::spawn(move || {
        // ВНИМАНИЕ: вызываем try_fold напрямую на КАНАЛЕ (rx), а не на итераторе!
        // Родной метод try_fold в kanal возвращает Result<Acc, ReceiveError>
        let mut agregated = Vec::with_capacity(rx.len());
        let r = rx.drain_into(&mut agregated);
        if r.is_err() {
            println!("Error: drain_into() вернул ошибку: {:?}", r);
        }
        agregated
    });
    // Даем потоку немного времени на выполнение
    std::thread::sleep(Duration::from_millis(200));

    if handle.is_finished() {
        // Поток успешно завершился сам, так как try_fold не блокирует поток
        let fold_res = handle.join().unwrap();
        println!("✅ Успех! drain_into() мгновенно вышел. Результат: {:?}", fold_res);
    } else {
        println!("try_recv КЛИНИТ!");
    }
}
