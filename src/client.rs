pub mod calculator {
    tonic::include_proto!("calculator");
}
use futures::stream;
use tonic::Request;
use calculator::calculator_client::CalculatorClient;
use calculator::{SumRequest, PrimeFactorRequest, AverageRequest, FindMaxRequest};

use std::time::Duration;
use tokio::time;

type Client = calculator::calculator_client::CalculatorClient<tonic::transport::Channel>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CalculatorClient::connect("http://[::1]:50051").await?;
    client_sum(&mut client, vec![]).await?;
    client_sum(&mut client, vec![2,4,6,8,-1,-3,-5,-7]).await?;
    client_sum(&mut client, vec![1,2,3,4,5,6,7,8,9]).await?;
    client_prime_factors(&mut client, 2*3*11*17*1001*4711).await?;
    client_prime_factors(&mut client, 0).await?;
    client_prime_factors(&mut client, 1).await?;
    client_average(&mut client, vec![1.0,4.0,9.0,16.0,25.0]).await?;
    client_find_max(&mut client, vec![3,5,8,5,3,2,9,5,6,7,11,4,7,98,7,99,5]).await?;
    Ok(())
}

async fn client_sum(client: &mut Client, data: Vec<i32>) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = time::interval(Duration::from_millis(300));
    println!("Request to rpc sum({:?})", data);
    let response = client.sum(Request::new(SumRequest {
        data: data.clone(),
    })).await?.into_inner();

    println!("Response from rpc sum({:?}): {}", data, response.sum);
    interval.tick().await;
    Ok(())
}

async fn client_prime_factors(client: &mut Client, number: u64) -> Result<(), Box<dyn std::error::Error>> {

    println!("Request to rpc prime_factors({})", number);

    let request = Request::new(PrimeFactorRequest{number});
    let mut stream = client.prime_factors(request).await?.into_inner();

    let mut factors = Vec::new();
    while let Some(response) = stream.message().await? {
        println!("factor: {:?}", response.factor);
        factors.push(response.factor);
    }
    println!("Response from rpc prime_factors({}): {:?}, with product: {}", number, factors, product(&factors));
    Ok(())
}

fn product(factors: &Vec<u64>) -> u64 {
    let mut prod = 1;
    for factor in factors.iter() {
        prod *= factor;
    }
    prod
}

async fn client_average(client: &mut Client, data: Vec<f64>) -> Result<(), Box<dyn std::error::Error>> {

    println!("Request to rpc average({:?})", data);
    let mut average_request = Vec::new();
    for x in data.iter() {
        average_request.push(AverageRequest{number:*x});
    }
    let request = Request::new(stream::iter(average_request));
    let response = client.average(request).await?.into_inner();

    println!("Response from rpc average({:?}): {:?}", data, response);
    Ok(())
}

async fn client_find_max(client: &mut Client, data: Vec<i32>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Request to rpc find_max({:?})", data);

    let outbound = async_stream::stream! {
        for x in data.iter() {
            let request = FindMaxRequest{number:*x};
            println!("{},{:?}",*x,request);
            yield request;
        }
    };
    let response = client.find_max(Request::new(outbound)).await?;
    let mut inbound = response.into_inner();

    while let Some(note) = inbound.message().await? {
        println!("Response from find_max: {:?}", note);
    }

    Ok(())
}