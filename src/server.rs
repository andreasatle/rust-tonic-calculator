use tonic::{transport::Server, Request, Response, Status, Streaming, Code};
use std::pin::Pin;
use calculator::calculator_server::{Calculator, CalculatorServer};
use calculator::{SumResponse, SumRequest};
use calculator::{PrimeFactorResponse, PrimeFactorRequest};
use calculator::{AverageResponse, AverageRequest};
use calculator::{FindMaxResponse, FindMaxRequest};
use calculator::{SquareRootResponse, SquareRootRequest};
use futures::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;
use tokio::sync::mpsc;

use std::time::Duration;
use tokio::time;

/// Include generated proto server and client items.
pub mod calculator {
    tonic::include_proto!("calculator");
}

/// MyCalculator implements Calculator.
/// If the server has state, then the state can be put here.
#[derive(Debug, Default)]
pub struct MyCalculator {}

#[tonic::async_trait]
impl Calculator for MyCalculator {
    /// rpc sum compute the sum of the input (that is a vector).
    async fn sum(
        &self,
        request: Request<SumRequest>,
    ) -> Result<Response<SumResponse>, Status> {
        let data = request.into_inner().data;
        println!("sum({:?})", data);

        let mut sum = 0;
        for num in data.iter() {
            sum += num;
        }

        let response = calculator::SumResponse {
            sum,
        };

        Ok(Response::new(response))
    }

    /// PrimeFactorsStream is the type of an output-stream of function prime_factors.
    type PrimeFactorsStream = ReceiverStream<Result<PrimeFactorResponse, Status>>;

    /// prime_factors return a stream of prime factors of the input upon success.
    async fn prime_factors(
        &self,
        request: Request<PrimeFactorRequest>,
    ) -> Result<Response<Self::PrimeFactorsStream>, Status> {
        let mut n = request.into_inner().number;
        println!("prime_factors({:?})", n);

        let mut interval = time::interval(Duration::from_millis(100));

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut k = 2;

            // A very primitive prime factor algorithm.
            // Special case for n = 0, and 1.
            if n < k {
                let response = calculator::PrimeFactorResponse {
                    factor: n,
                };
                tx.send(Ok(response)).await.unwrap();
                return;
            }
            // Case with one or more prime factors.
            while n >= k {
                if n%k == 0 {
                    let response = calculator::PrimeFactorResponse {
                        factor: k,
                    };
                    tx.send(Ok(response)).await.unwrap();
                    // Wait a bit to slow things down.
                    interval.tick().await;
                    n /= k;
                } else {
                    k += 1;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    /// average take a stream of numbers and return the average, when the stream is closed.
    async fn average(&self, request: Request<tonic::Streaming<AverageRequest>>) -> Result<Response<AverageResponse>, Status> {
        println!("average(stream)");
        let average = |sum:f64, cnt:i64| if cnt > 0 {sum / cnt as f64} else {0.0};
        let mut sum = 0.0;
        let mut cnt = 0;
        let mut stream = request.into_inner();
        while let Some(request) = stream.next().await {
            let request = request?; // Convert Ok(request) -> request...
            sum += request.number;
            cnt += 1;
            println!("average({}): {}", request.number, average(sum, cnt));
        }
        let response = calculator::AverageResponse {
            average: average(sum, cnt),
        };

        Ok(Response::new(response))
    }

    /// FindMaxStream is the type of an output-stream of function find_max.
    type FindMaxStream = Pin<Box<dyn Stream<Item = Result<FindMaxResponse, Status>> + Send + 'static>>;

    /// find_max takes a stream of input integers and return a stream of max-values when they show up.
    async fn find_max(&self, request: Request<Streaming<FindMaxRequest>>) -> Result<Response<Self::FindMaxStream>, Status> {
        println!("find_max(stream)");
        let mut interval = time::interval(Duration::from_millis(200));

        let mut max_val = 0;
        let mut first = true;

        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            while let Some(req) = stream.next().await {
                let req = req?;

                println!("find_max({}), with max: {}", req.number, max_val);
                if first || req.number > max_val {
                    max_val = req.number;
                    first = false;
                    interval.tick().await;

                    yield FindMaxResponse{max: max_val};
                }
            }
        };
        Ok(Response::new(Box::pin(output) as Self::FindMaxStream))

    }

    /// square_root compute the square root of an input argument.
    /// When the argument is negative, we return an error-message.
    async fn square_root(&self, request: Request<SquareRootRequest>) -> Result<Response<SquareRootResponse>, Status> {
        let number = request.into_inner().number;
        println!("square_root({:?})", number);

        if number < 0.0 {
            return Err(Status::new(Code::InvalidArgument, "Square root of negative number"));
        }
        
        let response = calculator::SquareRootResponse {
            sqrt: number.sqrt(),
        };

        Ok(Response::new(response))
    }

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;

    Server::builder()
        .add_service(CalculatorServer::new(MyCalculator::default()))
        .serve(addr)
        .await?;

    Ok(())
}