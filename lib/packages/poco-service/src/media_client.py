import asyncio
import grpc
import transcoding_pb2
import transcoding_pb2_grpc

from loguru import logger

# async def send_request(stub, name):


async def run() -> None:
    async with grpc.aio.insecure_channel("localhost:50051") as channel:
        stub = transcoding_pb2_grpc.TranscoderStub(channel)
        print("finish initialize")
        # 创建任务
        # response = await stub.DispatchVoDTask(transcoding_pb2.DispatchVoDRequest(
        #     taskid="100000000a",
        #     originurl="/home/wd/cartoon-swim.mp4",
        #     # outputurl="/home/wd/",
        #     outputcodec=1,
        #     # outputresolution=2,
        #     # outputaudiocodec=1,
        #     # outputframerate="24",
        #     # bitrate=3,
        #     # videoinfo=transcoding_pb2.VideoInfo(
        #     #     vid="12312312",
        #     #     duration="468.93",
        #     #     origincodec=1,
        #     #     originresolution=2,
        #     #     originaudiocodec=1,
        #     #     originframerate="24",
        #     #     originbitrate="180000"
        #     # )
        #     ))
        async for response in stub.DispatchVoDTask(
            transcoding_pb2.DispatchVoDRequest(
                taskid="100000000a",
                originurl="QmZZf7diD6jjJxKGipbKZ2d6W71u7FacTR8oN54rSMAcce",
                outputcodec=1,
            )
        ):
            logger.info(f"Received back {response}")


if __name__ == "__main__":
    asyncio.run(run())
