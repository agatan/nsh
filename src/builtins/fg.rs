use crate::builtins::InternalCommandContext;
use crate::exec::{ExitStatus, JobId, Job};
use structopt::StructOpt;
use std::io::Write;
use std::sync::Arc;

#[derive(Debug, StructOpt)]
#[structopt(name = "fg", about = "fg command.")]
struct Opt {
    #[structopt(name = "job_id")]
    job_id: Option<String>,
}

// Used by bg.
pub(super) fn parse_job_id(
    ctx: &mut InternalCommandContext,
    job_id: Option<String>
) -> Result<Arc<Job>, ExitStatus> {
    let id = match job_id {
        Some(job_id) => {
            let job_id = job_id.as_str();
            if job_id.chars().nth(0) == Some('%') {
                match (&job_id[1..]).parse() {
                    Ok(job_id) => JobId::new(job_id),
                    Err(_) => {
                        write!(ctx.stdout, "nsh: invalid job id `{}'\n", job_id).ok();
                        return Err(ExitStatus::ExitedWith(1));
                    },
                }
            } else {
                write!(ctx.stdout, "nsh: invalid job id `{}'\n", job_id).ok();
                return Err(ExitStatus::ExitedWith(1));
            }
        },
        None => {
            match ctx.isolate.last_fore_job() {
                Some(job) => job.id(),
                None => {
                    write!(ctx.stdout, "nsh: no jobs to run\n").ok();
                    return Err(ExitStatus::ExitedWith(1));
                }
            }
        }
    };

    match ctx.isolate.find_job_by_id(id) {
        Some(job) => Ok(job),
        None => {
            write!(ctx.stdout, "nsh: no such job `{}'\n", id).ok();
            return Err(ExitStatus::ExitedWith(1));
        }
    }
}

pub fn command(ctx: &mut InternalCommandContext) -> ExitStatus {
    trace!("fg: {:?}", ctx.argv);
    match Opt::from_iter_safe(ctx.argv) {
        Ok(opts) => {
            match parse_job_id(ctx, opts.job_id) {
                Ok(job) => {
                    ctx.isolate.continue_job(job, false);
                    ExitStatus::ExitedWith(0)
                },
                Err(status) => status,
            }
        },
        Err(err) => {
            write!(ctx.stdout, "fg: {}", err).ok();
            ExitStatus::ExitedWith(1)
        }
    }
}