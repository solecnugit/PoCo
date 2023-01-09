export class Segment{
    sequence_number: number;
    total_number: number;
    video_id: string;
    job_id: string;

    constructor(sequence: number, total: number, videoId: string, jobId: string){
        this.sequence_number = sequence;
        this.total_number = total;
        this.video_id = videoId;
        this.job_id = jobId;
    }
}