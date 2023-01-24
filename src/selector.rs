use crate::{job::Job, readable::Readable, schedule_queue::ScheduleQueue, Context};

pub async fn select_job(context: &mut Context) -> Job {
    if context.schedule_queue.is_first_urgent() {
        return context.schedule_queue.pop().unwrap().job;
    }

    tokio::select! {
        Ok((stream, addr)) = context.tcp_listener.accept() => {
            Job::AcceptFromTcp(stream, addr)
        }
        Ok(i) = context.waitings.readable() => {
            Job::ReadableFromWaiting(i)
        }
        Ok(addr) = context.tcp_streams.readable() => {
            Job::ReadableFromTcp(addr)
        }
        Ok(_) = context.udp_socket.readable() => {
            Job::ReadableFromUdp
        }
        Ok(_) = context.schedule_queue.wait_for_first() => {
            context.schedule_queue.pop().unwrap().job
        },
    }
}
