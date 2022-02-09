use tonic::{transport::Server, Request, Response, Status};

pub mod calculator {
    tonic::include_proto!("calculator");
}

use calculator::calculator_server::{Calculator, CalculatorServer};
use calculator::{SumResponse, SumRequest};
use calculator::{PrimeFactorResponse, PrimeFactorRequest};
use calculator::{AverageResponse, AverageRequest};
use futures::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc;

use std::time::Duration;
use tokio::time;

#[derive(Debug, Default)]
pub struct MyCalculator {}

#[tonic::async_trait]
impl Calculator for MyCalculator {
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

    type PrimeFactorsStream = ReceiverStream<Result<PrimeFactorResponse, Status>>;

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